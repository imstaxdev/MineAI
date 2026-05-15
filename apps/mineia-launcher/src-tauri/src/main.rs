#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, sync::Mutex};

use mineia_core::{CreateProfileRequest, MineiaContext, Profile, ProfileSettings};
use mineia_modpacks::{ImportedFile, ModpackImportReport};
use mineia_runtime::{InstallVersionReport, JavaInfo, LaunchResult};
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
fn detect_java() -> Result<JavaInfo, String> {
    mineia_runtime::detect_java().map_err(to_message)
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
fn launch_profile(state: State<'_, AppState>, profile_id: i64) -> Result<LaunchResult, String> {
    let context = lock_context(&state)?;
    let profile = context.store.get_profile(profile_id).map_err(to_message)?;
    mineia_runtime::launch_profile(&context.paths, &profile).map_err(to_message)
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
            detect_java,
            install_version,
            launch_profile,
            import_mod,
            import_shader,
            import_modpack,
            repair_profile
        ])
        .run(tauri::generate_context!())
        .expect("Error ejecutando MineIA.");
}
