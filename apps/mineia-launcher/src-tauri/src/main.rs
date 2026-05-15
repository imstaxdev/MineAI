#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
};

use mineia_core::{CreateProfileRequest, MineiaContext, Profile, ProfileSettings};
use mineia_modpacks::{ImportedFile, ModpackImportReport};
use mineia_runtime::{
    InstallVersionReport, JavaInfo, LaunchResult, MinecraftVersionItem, MineiaGameOptions,
};
use serde::Serialize;
use tauri::State;

struct AppState {
    context: Mutex<MineiaContext>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepairReport {
    profile_id: i64,
    ensured_directories: Vec<String>,
}

#[tauri::command]
fn create_profile(state: State<'_, AppState>, username: String) -> Result<Profile, String> {
    let context = lock_context(&state)?;
    context
        .store
        .create_profile(CreateProfileRequest { username })
        .map_err(to_message)
}

#[tauri::command]
fn list_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let context = lock_context(&state)?;
    context.store.list_profiles().map_err(to_message)
}

#[tauri::command]
fn update_profile_settings(
    state: State<'_, AppState>,
    profile_id: i64,
    settings: ProfileSettings,
) -> Result<Profile, String> {
    let context = lock_context(&state)?;
    context
        .store
        .update_profile_settings(profile_id, settings)
        .map_err(to_message)
}

#[tauri::command]
fn get_mineia_options(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<MineiaGameOptions, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    mineia_runtime::load_mineia_options(&context.paths.profile_dir(profile.id)).map_err(to_message)
}

#[tauri::command]
fn save_mineia_options(
    state: State<'_, AppState>,
    profile_id: i64,
    options: MineiaGameOptions,
) -> Result<MineiaGameOptions, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    let profile_dir = context.paths.profile_dir(profile.id);
    mineia_runtime::save_mineia_options(&profile_dir, options).map_err(to_message)?;
    mineia_runtime::load_mineia_options(&profile_dir).map_err(to_message)
}

#[tauri::command]
fn detect_java() -> Result<JavaInfo, String> {
    mineia_runtime::detect_java().map_err(to_message)
}

#[tauri::command]
fn is_version_installed(state: State<'_, AppState>, version: String) -> Result<bool, String> {
    let context = lock_context(&state)?;
    Ok(mineia_runtime::is_version_installed(
        &context.paths,
        &version,
    ))
}

#[tauri::command]
async fn list_minecraft_versions(
    state: State<'_, AppState>,
) -> Result<Vec<MinecraftVersionItem>, String> {
    let paths = {
        let context = lock_context(&state)?;
        context.paths.clone()
    };
    mineia_runtime::list_minecraft_versions(&paths, 32)
        .await
        .map_err(to_message)
}

#[tauri::command]
async fn install_version(
    state: State<'_, AppState>,
    version: String,
) -> Result<InstallVersionReport, String> {
    let paths = {
        let context = lock_context(&state)?;
        context.paths.clone()
    };
    mineia_runtime::install_vanilla_version(&paths, &version)
        .await
        .map_err(to_message)
}

#[tauri::command]
async fn prepare_version_for_launch(
    state: State<'_, AppState>,
    version: String,
) -> Result<InstallVersionReport, String> {
    let paths = {
        let context = lock_context(&state)?;
        context.paths.clone()
    };
    mineia_runtime::prepare_version_for_launch(&paths, &version)
        .await
        .map_err(to_message)
}

#[tauri::command]
fn launch_profile(state: State<'_, AppState>, profile_id: i64) -> Result<LaunchResult, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    mineia_runtime::launch_profile(&context.paths, &profile).map_err(to_message)
}

#[tauri::command]
fn read_log_tail(
    state: State<'_, AppState>,
    log_file: String,
    max_bytes: Option<u64>,
) -> Result<String, String> {
    let context = lock_context(&state)?;
    let log_path = PathBuf::from(log_file);
    ensure_log_path(&context.paths.logs_dir, &log_path)?;

    let mut file = fs::File::open(log_path).map_err(to_message)?;
    let len = file.metadata().map_err(to_message)?.len();
    let read_len = max_bytes.unwrap_or(32 * 1024).clamp(1024, 256 * 1024);
    let start = len.saturating_sub(read_len);
    file.seek(SeekFrom::Start(start)).map_err(to_message)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(to_message)?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

#[tauri::command]
fn import_mod(
    state: State<'_, AppState>,
    profile_id: i64,
    file: String,
) -> Result<ImportedFile, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    mineia_modpacks::import_mod(&context.paths, &profile, file).map_err(to_message)
}

#[tauri::command]
fn import_shader(
    state: State<'_, AppState>,
    profile_id: i64,
    file: String,
) -> Result<ImportedFile, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    mineia_modpacks::import_shader(&context.paths, &profile, file).map_err(to_message)
}

#[tauri::command]
fn import_modpack(
    state: State<'_, AppState>,
    profile_id: i64,
    file: String,
) -> Result<ModpackImportReport, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    mineia_modpacks::import_modpack(&context.paths, &profile, file).map_err(to_message)
}

#[tauri::command]
fn repair_profile(state: State<'_, AppState>, profile_id: i64) -> Result<RepairReport, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    let profile_dir = context.paths.profile_dir(profile.id);
    let dirs = [
        profile_dir.clone(),
        profile_dir.join("mods"),
        profile_dir.join("shaderpacks"),
        profile_dir.join("resourcepacks"),
        profile_dir.join("config"),
        profile_dir.join("crash-reports"),
        context.paths.logs_dir.clone(),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir).map_err(to_message)?;
    }

    Ok(RepairReport {
        profile_id,
        ensured_directories: dirs
            .into_iter()
            .map(|dir| dir.display().to_string())
            .collect(),
    })
}

fn lock_context<'a>(
    state: &'a State<'_, AppState>,
) -> Result<std::sync::MutexGuard<'a, MineiaContext>, String> {
    state
        .context
        .lock()
        .map_err(|_| "No se pudo bloquear el estado local de MineIA.".to_owned())
}

fn to_message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn ensure_log_path(logs_dir: &Path, log_path: &Path) -> Result<(), String> {
    let canonical_logs = logs_dir
        .canonicalize()
        .map_err(|_| "No se encontro la carpeta de logs de MineIA.".to_owned())?;
    let canonical_log = log_path
        .canonicalize()
        .map_err(|_| "No se encontro el log solicitado.".to_owned())?;

    if canonical_log.starts_with(canonical_logs) {
        Ok(())
    } else {
        Err("Ruta de log no permitida.".to_owned())
    }
}

fn main() {
    let context = MineiaContext::open_default()
        .expect("No se pudo inicializar MineIA. Revisa permisos de la carpeta de datos local.");

    tauri::Builder::default()
        .manage(AppState {
            context: Mutex::new(context),
        })
        .invoke_handler(tauri::generate_handler![
            create_profile,
            list_profiles,
            update_profile_settings,
            get_mineia_options,
            save_mineia_options,
            detect_java,
            is_version_installed,
            list_minecraft_versions,
            install_version,
            prepare_version_for_launch,
            launch_profile,
            read_log_tail,
            import_mod,
            import_shader,
            import_modpack,
            repair_profile
        ])
        .run(tauri::generate_context!())
        .expect("Error ejecutando MineIA.");
}
