use std::{
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
};

use mineia_core::{hash_file, FileHashes, MineiaPaths, Profile};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::ZipArchive;

const MAX_OVERRIDE_FILE_SIZE: u64 = 256 * 1024 * 1024;
const MAX_OVERRIDE_TOTAL_SIZE: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum ModpackError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Core(#[from] mineia_core::CoreError),
    #[error("Tipo de archivo no soportado: {0}")]
    UnsupportedFile(String),
    #[error("El archivo intenta escribir fuera del perfil: {0}")]
    UnsafeArchivePath(String),
    #[error("El archivo del modpack supera el limite permitido: {path} ({size} bytes)")]
    ArchiveEntryTooLarge { path: String, size: u64 },
    #[error("Los overrides del modpack superan el limite total permitido.")]
    ArchiveTooLarge,
}

pub type ModpackResult<T> = Result<T, ModpackError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedFile {
    pub file_name: String,
    pub destination: PathBuf,
    pub hashes: FileHashes,
    pub reused_existing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackImportReport {
    pub name: String,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub files_declared: usize,
    pub downloads_required: usize,
    pub overrides_extracted: usize,
}

#[derive(Debug, Deserialize)]
struct MrpackManifest {
    name: String,
    #[serde(default)]
    dependencies: std::collections::HashMap<String, String>,
    #[serde(default)]
    files: Vec<MrpackFile>,
}

#[derive(Debug, Deserialize)]
struct MrpackFile {
    path: String,
    #[serde(default)]
    downloads: Vec<String>,
}

pub fn import_mod(
    paths: &MineiaPaths,
    profile: &Profile,
    source: impl AsRef<Path>,
) -> ModpackResult<ImportedFile> {
    copy_profile_file(paths, profile, source, "mods", &["jar"])
}

pub fn import_shader(
    paths: &MineiaPaths,
    profile: &Profile,
    source: impl AsRef<Path>,
) -> ModpackResult<ImportedFile> {
    copy_profile_file(paths, profile, source, "shaderpacks", &["zip"])
}

pub fn import_modpack(
    paths: &MineiaPaths,
    profile: &Profile,
    source: impl AsRef<Path>,
) -> ModpackResult<ModpackImportReport> {
    let source = source.as_ref();
    let extension = extension_lowercase(source);
    match extension.as_deref() {
        Some("mrpack") => import_mrpack(paths, profile, source),
        Some("zip") => {
            fs::create_dir_all(paths.profile_dir(profile.id).join("modpacks"))?;
            let imported = copy_profile_file(paths, profile, source, "modpacks", &["zip"])?;
            Ok(ModpackImportReport {
                name: imported.file_name,
                game_version: None,
                loader: None,
                files_declared: 0,
                downloads_required: 0,
                overrides_extracted: 0,
            })
        }
        _ => Err(ModpackError::UnsupportedFile(source.display().to_string())),
    }
}

fn copy_profile_file(
    paths: &MineiaPaths,
    profile: &Profile,
    source: impl AsRef<Path>,
    folder: &str,
    allowed_extensions: &[&str],
) -> ModpackResult<ImportedFile> {
    let source = source.as_ref();
    let extension = extension_lowercase(source).unwrap_or_default();
    if !allowed_extensions
        .iter()
        .any(|allowed| *allowed == extension)
    {
        return Err(ModpackError::UnsupportedFile(source.display().to_string()));
    }

    let file_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ModpackError::UnsupportedFile(source.display().to_string()))?
        .to_owned();

    let destination_dir = paths.profile_dir(profile.id).join(folder);
    fs::create_dir_all(&destination_dir)?;
    let destination = destination_dir.join(&file_name);
    let source_hashes = hash_file(source)?;

    let reused_existing = if destination.is_file() {
        hash_file(&destination)?.sha256 == source_hashes.sha256
    } else {
        false
    };

    if !reused_existing {
        fs::copy(source, &destination)?;
    }

    Ok(ImportedFile {
        file_name,
        destination,
        hashes: source_hashes,
        reused_existing,
    })
}

fn import_mrpack(
    paths: &MineiaPaths,
    profile: &Profile,
    source: &Path,
) -> ModpackResult<ModpackImportReport> {
    let file = File::open(source)?;
    let mut archive = ZipArchive::new(file)?;
    let manifest = read_mrpack_manifest(&mut archive)?;
    let profile_dir = paths.profile_dir(profile.id);
    fs::create_dir_all(&profile_dir)?;

    let mut overrides_extracted = 0;
    let mut total_override_size = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let Some(relative) = name.strip_prefix("overrides/") else {
            continue;
        };
        if relative.is_empty() || entry.is_dir() {
            continue;
        }

        if entry.size() > MAX_OVERRIDE_FILE_SIZE {
            return Err(ModpackError::ArchiveEntryTooLarge {
                path: name,
                size: entry.size(),
            });
        }
        total_override_size = total_override_size.saturating_add(entry.size());
        if total_override_size > MAX_OVERRIDE_TOTAL_SIZE {
            return Err(ModpackError::ArchiveTooLarge);
        }

        let safe_relative = safe_relative_path(relative)?;
        let destination = profile_dir.join(safe_relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output = File::create(destination)?;
        io::copy(&mut entry, &mut output)?;
        overrides_extracted += 1;
    }

    let game_version = manifest.dependencies.get("minecraft").cloned();
    let loader = manifest
        .dependencies
        .iter()
        .find(|(key, _)| {
            matches!(
                key.as_str(),
                "fabric-loader" | "quilt-loader" | "forge" | "neoforge"
            )
        })
        .map(|(key, version)| format!("{key}:{version}"));

    let downloads_required = manifest
        .files
        .iter()
        .filter(|file| !file.downloads.is_empty() && !profile_dir.join(&file.path).is_file())
        .count();

    Ok(ModpackImportReport {
        name: manifest.name,
        game_version,
        loader,
        files_declared: manifest.files.len(),
        downloads_required,
        overrides_extracted,
    })
}

fn read_mrpack_manifest(archive: &mut ZipArchive<File>) -> ModpackResult<MrpackManifest> {
    let manifest = archive.by_name("modrinth.index.json")?;
    Ok(serde_json::from_reader(manifest)?)
}

fn safe_relative_path(path: &str) -> ModpackResult<PathBuf> {
    let mut safe = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => safe.push(part),
            _ => return Err(ModpackError::UnsafeArchivePath(path.to_owned())),
        }
    }
    Ok(safe)
}

fn extension_lowercase(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
}
