use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSettings {
    pub minecraft_version: Option<String>,
    pub ram_mb: Option<u32>,
    pub java_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: i64,
    pub username: String,
    pub offline_uuid: String,
    pub minecraft_version: String,
    pub ram_mb: u32,
    pub java_path: Option<PathBuf>,
    pub created_at: String,
    pub updated_at: String,
}

impl Profile {
    pub fn auth_uuid(&self) -> String {
        self.offline_uuid.clone()
    }

    pub fn auth_access_token(&self) -> String {
        "0".to_owned()
    }

    pub fn user_type(&self) -> &'static str {
        "legacy"
    }
}
