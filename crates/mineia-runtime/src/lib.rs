use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

use chrono::Utc;
use mineia_core::{MineiaPaths, Profile};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use thiserror::Error;
#[cfg(windows)]
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{SetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS};
use zip::ZipArchive;

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x00000008;
#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Core(#[from] mineia_core::CoreError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error("No se encontro Java. Instala Java 17+ o configura una ruta manual.")]
    JavaNotFound,
    #[error("Minecraft {0} no existe en el manifest oficial.")]
    VersionNotFound(String),
    #[error("No existe el jar de Minecraft para la version {version}: {path}")]
    MissingMinecraftJar { version: String, path: String },
    #[error("El hash no coincide para {path}. Esperado {expected}, obtenido {actual}.")]
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInfo {
    pub path: PathBuf,
    pub version: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub pid: u32,
    pub log_file: PathBuf,
    pub command_preview: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallVersionReport {
    pub version: String,
    pub downloaded_files: usize,
    pub reused_files: usize,
    pub version_dir: PathBuf,
    pub libraries: usize,
    pub assets: usize,
    pub natives: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersionItem {
    pub id: String,
    pub version_type: String,
    pub latest: bool,
    pub installed: bool,
}

#[derive(Debug, Deserialize)]
struct VersionManifest {
    latest: VersionManifestLatest,
    versions: Vec<VersionManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct VersionManifestLatest {
    release: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionManifestEntry {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionJson {
    id: String,
    #[serde(default)]
    assets: String,
    #[serde(default)]
    main_class: String,
    #[serde(default)]
    arguments: VersionArguments,
    #[serde(default)]
    minecraft_arguments: Option<String>,
    asset_index: AssetIndexRef,
    downloads: VersionDownloads,
    #[serde(default)]
    libraries: Vec<Library>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct VersionArguments {
    #[serde(default)]
    game: Vec<ArgumentItem>,
    #[serde(default)]
    jvm: Vec<ArgumentItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum ArgumentItem {
    Raw(String),
    Ruled {
        #[serde(default)]
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum ArgumentValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Rule {
    action: RuleAction,
    #[serde(default)]
    os: Option<RuleOs>,
    #[serde(default)]
    features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RuleOs {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetIndexRef {
    id: String,
    url: String,
    sha1: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VersionDownloads {
    client: DownloadInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DownloadInfo {
    #[serde(default)]
    path: Option<String>,
    url: String,
    #[serde(default)]
    sha1: Option<String>,
    #[serde(default)]
    size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Library {
    name: String,
    #[serde(default)]
    downloads: LibraryDownloads,
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(default)]
    natives: HashMap<String, String>,
    #[serde(default)]
    extract: Option<LibraryExtract>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct LibraryDownloads {
    #[serde(default)]
    artifact: Option<DownloadInfo>,
    #[serde(default)]
    classifiers: HashMap<String, DownloadInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LibraryExtract {
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
struct AssetObject {
    hash: String,
    #[serde(default)]
    size: Option<u64>,
}

pub fn detect_java() -> RuntimeResult<JavaInfo> {
    let mut candidates = Vec::new();

    if let Ok(path) = env::var("MINEIA_JAVA_HOME") {
        candidates.push((
            PathBuf::from(path).join("bin").join(java_bin_name()),
            "MINEIA_JAVA_HOME",
        ));
    }

    if let Ok(path) = env::var("JAVA_HOME") {
        candidates.push((
            PathBuf::from(path).join("bin").join(java_bin_name()),
            "JAVA_HOME",
        ));
    }

    if let Some(path) = find_java_on_path() {
        candidates.push((path, "PATH"));
    }

    for (path, source) in candidates {
        if path.is_file() {
            return Ok(JavaInfo {
                version: read_java_version(&path),
                path,
                source: source.to_owned(),
            });
        }
    }

    Err(RuntimeError::JavaNotFound)
}

pub async fn install_vanilla_version(
    paths: &MineiaPaths,
    version: &str,
) -> RuntimeResult<InstallVersionReport> {
    install_version_files(paths, version, true).await
}

pub async fn prepare_version_for_launch(
    paths: &MineiaPaths,
    version: &str,
) -> RuntimeResult<InstallVersionReport> {
    install_version_files(paths, version, false).await
}

async fn install_version_files(
    paths: &MineiaPaths,
    version: &str,
    include_assets: bool,
) -> RuntimeResult<InstallVersionReport> {
    paths.ensure_runtime_dirs()?;

    let client = http_client()?;
    let manifest: VersionManifest = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let entry = manifest
        .versions
        .into_iter()
        .find(|entry| entry.id == version)
        .ok_or_else(|| RuntimeError::VersionNotFound(version.to_owned()))?;

    let version_json: VersionJson = client
        .get(entry.url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let version_dir = paths.versions_dir.join(&version_json.id);
    fs::create_dir_all(&version_dir)?;
    let version_json_path = version_dir.join(format!("{}.json", version_json.id));
    fs::write(
        &version_json_path,
        serde_json::to_vec_pretty(&version_json)?,
    )?;

    let mut report = InstallVersionReport {
        version: version_json.id.clone(),
        downloaded_files: 0,
        reused_files: 0,
        version_dir: version_dir.clone(),
        libraries: 0,
        assets: 0,
        natives: 0,
    };

    download_verified(
        &client,
        &version_json.downloads.client.url,
        &version_dir.join(format!("{}.jar", version_json.id)),
        version_json.downloads.client.sha1.as_deref(),
        version_json.downloads.client.size,
        &mut report,
    )
    .await?;

    let indexes_dir = paths.assets_dir.join("indexes");
    fs::create_dir_all(&indexes_dir)?;
    let asset_index_path = indexes_dir.join(format!("{}.json", version_json.asset_index.id));
    download_verified(
        &client,
        &version_json.asset_index.url,
        &asset_index_path,
        Some(&version_json.asset_index.sha1),
        None,
        &mut report,
    )
    .await?;

    let natives_dir = paths.natives_dir(&version_json.id);
    if natives_dir.exists() {
        fs::remove_dir_all(&natives_dir)?;
    }
    fs::create_dir_all(&natives_dir)?;

    for library in version_json
        .libraries
        .iter()
        .filter(|library| rules_allow(&library.rules))
    {
        if let Some(artifact) = &library.downloads.artifact {
            let destination = paths.libraries_dir.join(
                artifact
                    .path
                    .clone()
                    .unwrap_or_else(|| maven_path_from_name(&library.name, "jar")),
            );
            download_verified(
                &client,
                &artifact.url,
                &destination,
                artifact.sha1.as_deref(),
                artifact.size,
                &mut report,
            )
            .await?;
            report.libraries += 1;

            if is_modern_windows_native(&library.name) {
                extract_native(&destination, &natives_dir, library.extract.as_ref())?;
                report.natives += 1;
            }
        }

        if let Some(native) = native_download(library) {
            let destination = paths.libraries_dir.join(
                native
                    .path
                    .clone()
                    .unwrap_or_else(|| maven_path_from_name(&library.name, "jar")),
            );
            download_verified(
                &client,
                &native.url,
                &destination,
                native.sha1.as_deref(),
                native.size,
                &mut report,
            )
            .await?;
            extract_native(&destination, &natives_dir, library.extract.as_ref())?;
            report.natives += 1;
        }
    }

    if include_assets {
        let asset_index: AssetIndex = serde_json::from_reader(File::open(&asset_index_path)?)?;
        for object in asset_index.objects.values() {
            let prefix = &object.hash[0..2];
            let destination = paths
                .assets_dir
                .join("objects")
                .join(prefix)
                .join(&object.hash);
            let url = format!(
                "https://resources.download.minecraft.net/{prefix}/{}",
                object.hash
            );
            download_verified(
                &client,
                &url,
                &destination,
                Some(&object.hash),
                object.size,
                &mut report,
            )
            .await?;
            report.assets += 1;
        }
    }

    Ok(report)
}

pub async fn list_minecraft_versions(
    paths: &MineiaPaths,
    limit: usize,
) -> RuntimeResult<Vec<MinecraftVersionItem>> {
    let client = http_client()?;
    let manifest: VersionManifest = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(manifest
        .versions
        .into_iter()
        .filter(|entry| entry.version_type == "release")
        .take(limit)
        .map(|entry| MinecraftVersionItem {
            latest: entry.id == manifest.latest.release,
            installed: is_version_installed(paths, &entry.id),
            version_type: entry.version_type,
            id: entry.id,
        })
        .collect())
}

pub fn is_version_installed(paths: &MineiaPaths, version: &str) -> bool {
    let version_dir = paths.versions_dir.join(version);
    version_dir.join(format!("{version}.jar")).is_file()
        && version_dir.join(format!("{version}.json")).is_file()
}

fn http_client() -> RuntimeResult<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?)
}

pub fn launch_profile(paths: &MineiaPaths, profile: &Profile) -> RuntimeResult<LaunchResult> {
    let java = if let Some(path) = &profile.java_path {
        JavaInfo {
            path: path.clone(),
            version: read_java_version(path),
            source: "manual".to_owned(),
        }
    } else {
        detect_java()?
    };

    let profile_dir = paths.profile_dir(profile.id);
    let version_dir = paths.versions_dir.join(&profile.minecraft_version);
    let version_jar = version_dir.join(format!("{}.jar", profile.minecraft_version));
    let version_json_path = version_dir.join(format!("{}.json", profile.minecraft_version));

    if !version_jar.is_file() {
        return Err(RuntimeError::MissingMinecraftJar {
            version: profile.minecraft_version.clone(),
            path: version_jar.display().to_string(),
        });
    }

    fs::create_dir_all(&profile_dir)?;
    fs::create_dir_all(&paths.logs_dir)?;
    write_lightweight_options(&profile_dir)?;

    let log_file = paths.logs_dir.join(format!(
        "profile-{}-{}.log",
        profile.id,
        Utc::now().format("%Y%m%d-%H%M%S")
    ));

    let version_json: VersionJson = serde_json::from_reader(File::open(version_json_path)?)?;
    let mut args = performance_jvm_args(automatic_ram_mb(paths, profile));
    args.extend(minecraft_args(paths, profile, &version_json, &version_jar));

    let stdout = File::create(&log_file)?;
    let stderr = stdout.try_clone()?;
    let mut command = Command::new(&java.path);
    command
        .args(&args)
        .current_dir(&profile_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));

    #[cfg(windows)]
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

    let child = command.spawn()?;
    set_low_priority(&child);

    let mut command_preview = Vec::with_capacity(args.len() + 1);
    command_preview.push(java.path.display().to_string());
    command_preview.extend(args);

    Ok(LaunchResult {
        pid: child.id(),
        log_file,
        command_preview,
    })
}

fn automatic_ram_mb(paths: &MineiaPaths, profile: &Profile) -> u32 {
    let mod_count = paths
        .profile_dir(profile.id)
        .join("mods")
        .read_dir()
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"))
                })
                .count()
        })
        .unwrap_or(0);

    let target = match mod_count {
        0 => 1536,
        1..=20 => 2048,
        21..=80 => 3072,
        _ => 4096,
    };
    match system_memory_mb() {
        Some(total) if total < 4096 => 1024,
        Some(total) => target.min((total / 4).max(1024)),
        None => target,
    }
}

fn performance_jvm_args(max_ram_mb: u32) -> Vec<String> {
    vec![
        format!("-Xmx{max_ram_mb}M"),
        "-Xms512M".to_owned(),
        "-XX:+UseG1GC".to_owned(),
        "-XX:+UnlockExperimentalVMOptions".to_owned(),
        "-XX:G1NewSizePercent=20".to_owned(),
        "-XX:G1ReservePercent=20".to_owned(),
        "-XX:MaxGCPauseMillis=50".to_owned(),
        "-XX:G1HeapRegionSize=16M".to_owned(),
        "-XX:+DisableExplicitGC".to_owned(),
        "-XX:+PerfDisableSharedMem".to_owned(),
        "-Dsun.rmi.dgc.server.gcInterval=2147483646".to_owned(),
    ]
}

#[cfg(windows)]
fn system_memory_mb() -> Option<u32> {
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };

    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    if ok == 0 {
        return None;
    }

    Some((status.ullTotalPhys / 1024 / 1024) as u32)
}

#[cfg(not(windows))]
fn system_memory_mb() -> Option<u32> {
    None
}

fn write_lightweight_options(profile_dir: &Path) -> RuntimeResult<()> {
    let options_path = profile_dir.join("options.txt");
    let mut lines = if options_path.is_file() {
        fs::read_to_string(&options_path)?
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let light_options = [
        ("enableVsync", "false"),
        ("maxFps", "60"),
        ("graphicsMode", "0"),
        ("renderDistance", "5"),
        ("simulationDistance", "3"),
        ("entityDistanceScaling", "0.5"),
        ("entityShadows", "false"),
        ("renderClouds", "\"false\""),
        ("cloudRange", "32"),
        ("biomeBlendRadius", "0"),
        ("mipmapLevels", "0"),
        ("particles", "2"),
        ("fovEffectScale", "0.0"),
        ("screenEffectScale", "0.0"),
        ("prioritizeChunkUpdates", "2"),
        ("inactivityFpsLimit", "\"minimized\""),
    ];

    for (key, value) in light_options {
        upsert_option(&mut lines, key, value);
    }

    fs::write(options_path, format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn upsert_option(lines: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{key}:");
    if let Some(line) = lines.iter_mut().find(|line| line.starts_with(&prefix)) {
        *line = format!("{key}:{value}");
    } else {
        lines.push(format!("{key}:{value}"));
    }
}

#[cfg(windows)]
fn set_low_priority(child: &std::process::Child) {
    unsafe {
        let _ = SetPriorityClass(child.as_raw_handle(), BELOW_NORMAL_PRIORITY_CLASS);
    }
}

#[cfg(not(windows))]
fn set_low_priority(_child: &std::process::Child) {}

fn minecraft_args(
    paths: &MineiaPaths,
    profile: &Profile,
    version_json: &VersionJson,
    version_jar: &Path,
) -> Vec<String> {
    let profile_dir = paths.profile_dir(profile.id);
    let natives_dir = paths.natives_dir(&version_json.id);
    let classpath = classpath(paths, version_json, version_jar);

    let replacements = HashMap::from([
        ("auth_player_name", profile.username.clone()),
        ("version_name", version_json.id.clone()),
        ("game_directory", profile_dir.display().to_string()),
        ("assets_root", paths.assets_dir.display().to_string()),
        ("assets_index_name", version_json.asset_index.id.clone()),
        ("auth_uuid", profile.auth_uuid()),
        ("auth_access_token", profile.auth_access_token()),
        ("user_type", profile.user_type().to_owned()),
        ("version_type", "release".to_owned()),
        ("natives_directory", natives_dir.display().to_string()),
        ("launcher_name", "MineIA".to_owned()),
        ("launcher_version", env!("CARGO_PKG_VERSION").to_owned()),
        ("classpath", classpath),
        (
            "classpath_separator",
            if cfg!(windows) { ";" } else { ":" }.to_owned(),
        ),
    ]);

    let mut args = expand_arguments(&version_json.arguments.jvm, &replacements);
    if args.is_empty() {
        args.extend([
            format!("-Djava.library.path={}", natives_dir.display()),
            "-cp".to_owned(),
            replacements["classpath"].clone(),
        ]);
    }

    args.push(version_json.main_class.clone());

    let mut game_args = expand_arguments(&version_json.arguments.game, &replacements);
    if game_args.is_empty() {
        game_args = legacy_minecraft_arguments(version_json, &replacements);
    }
    args.extend(game_args);

    args
}

fn classpath(paths: &MineiaPaths, version_json: &VersionJson, version_jar: &Path) -> String {
    let mut entries: Vec<String> = version_json
        .libraries
        .iter()
        .filter(|library| rules_allow(&library.rules))
        .filter_map(|library| library.downloads.artifact.as_ref())
        .map(|artifact| {
            paths
                .libraries_dir
                .join(
                    artifact
                        .path
                        .clone()
                        .unwrap_or_else(|| "missing-artifact.jar".to_owned()),
                )
                .display()
                .to_string()
        })
        .collect();

    entries.push(version_jar.display().to_string());
    entries.join(if cfg!(windows) { ";" } else { ":" })
}

fn expand_arguments(items: &[ArgumentItem], replacements: &HashMap<&str, String>) -> Vec<String> {
    let mut output = Vec::new();
    for item in items {
        match item {
            ArgumentItem::Raw(value) => output.push(replace_tokens(value, replacements)),
            ArgumentItem::Ruled { rules, value } if rules_allow(rules) => match value {
                ArgumentValue::One(item) => output.push(replace_tokens(item, replacements)),
                ArgumentValue::Many(items) => output.extend(
                    items
                        .iter()
                        .map(|item| replace_tokens(item, replacements))
                        .collect::<Vec<_>>(),
                ),
            },
            ArgumentItem::Ruled { .. } => {}
        }
    }
    output
}

fn legacy_minecraft_arguments(
    version_json: &VersionJson,
    replacements: &HashMap<&str, String>,
) -> Vec<String> {
    version_json
        .minecraft_arguments
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .map(|part| replace_tokens(part, replacements))
        .collect()
}

fn replace_tokens(input: &str, replacements: &HashMap<&str, String>) -> String {
    let mut output = input.to_owned();
    for (key, value) in replacements {
        output = output.replace(&format!("${{{key}}}"), value);
    }
    output
}

fn rules_allow(rules: &[Rule]) -> bool {
    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule) {
            allowed = rule.action == RuleAction::Allow;
        }
    }
    allowed
}

fn rule_matches(rule: &Rule) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != "windows" {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            if arch == "x86" && !cfg!(target_arch = "x86") {
                return false;
            }
        }
    }

    if let Some(features) = &rule.features {
        if features.values().any(|enabled| *enabled) {
            return false;
        }
    }

    true
}

fn native_download(library: &Library) -> Option<&DownloadInfo> {
    let classifier = library.natives.get("windows").map(|value| {
        value.replace(
            "${arch}",
            if cfg!(target_pointer_width = "64") {
                "64"
            } else {
                "32"
            },
        )
    });
    classifier
        .as_deref()
        .and_then(|classifier| library.downloads.classifiers.get(classifier))
}

fn is_modern_windows_native(name: &str) -> bool {
    if !name.contains(":natives-windows") {
        return false;
    }

    if name.ends_with(":natives-windows-arm64") {
        return cfg!(target_arch = "aarch64");
    }

    if name.ends_with(":natives-windows-x86") {
        return cfg!(target_arch = "x86");
    }

    true
}

async fn download_verified(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    expected_sha1: Option<&str>,
    expected_size: Option<u64>,
    report: &mut InstallVersionReport,
) -> RuntimeResult<()> {
    if destination_is_valid(destination, expected_sha1, expected_size)? {
        report.reused_files += 1;
        return Ok(());
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    fs::write(destination, &bytes)?;

    verify_file(destination, expected_sha1, expected_size)?;
    report.downloaded_files += 1;
    Ok(())
}

fn destination_is_valid(
    destination: &Path,
    expected_sha1: Option<&str>,
    expected_size: Option<u64>,
) -> RuntimeResult<bool> {
    if !destination.is_file() {
        return Ok(false);
    }
    match verify_file(destination, expected_sha1, expected_size) {
        Ok(()) => Ok(true),
        Err(RuntimeError::HashMismatch { .. }) => Ok(false),
        Err(error) => Err(error),
    }
}

fn verify_file(
    destination: &Path,
    expected_sha1: Option<&str>,
    expected_size: Option<u64>,
) -> RuntimeResult<()> {
    if let Some(expected_size) = expected_size {
        if destination.metadata()?.len() != expected_size {
            return Err(RuntimeError::HashMismatch {
                path: destination.display().to_string(),
                expected: expected_size.to_string(),
                actual: destination.metadata()?.len().to_string(),
            });
        }
    }

    if let Some(expected_sha1) = expected_sha1 {
        let actual = sha1_file(destination)?;
        if actual != expected_sha1 {
            return Err(RuntimeError::HashMismatch {
                path: destination.display().to_string(),
                expected: expected_sha1.to_owned(),
                actual,
            });
        }
    }

    Ok(())
}

fn sha1_file(path: &Path) -> RuntimeResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn extract_native(
    archive_path: &Path,
    natives_dir: &Path,
    extract: Option<&LibraryExtract>,
) -> RuntimeResult<()> {
    let mut archive = ZipArchive::new(File::open(archive_path)?)?;
    let excluded = extract
        .map(|extract| extract.exclude.as_slice())
        .unwrap_or(&[]);

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name();
        if entry.is_dir()
            || name.contains("..")
            || excluded.iter().any(|prefix| name.starts_with(prefix))
        {
            continue;
        }

        let destination = natives_dir.join(name);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = File::create(destination)?;
        io::copy(&mut entry, &mut output)?;
    }

    Ok(())
}

fn maven_path_from_name(name: &str, extension: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return format!("{}.{}", name.replace(':', "/"), extension);
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = parts
        .get(3)
        .map(|value| format!("-{value}"))
        .unwrap_or_default();
    format!("{group}/{artifact}/{version}/{artifact}-{version}{classifier}.{extension}")
}

fn read_java_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("-version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stderr);
    text.lines().next().map(|line| line.trim().to_owned())
}

fn find_java_on_path() -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;
    env::split_paths(&path_env)
        .map(|path| path.join(java_bin_name()))
        .find(|candidate| candidate.is_file())
}

fn java_bin_name() -> &'static str {
    if cfg!(windows) {
        "java.exe"
    } else {
        "java"
    }
}
