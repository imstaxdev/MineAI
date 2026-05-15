use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MineiaPaths {
    pub data_dir: PathBuf,
    pub database_file: PathBuf,
    pub downloads_dir: PathBuf,
    pub versions_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl MineiaPaths {
    pub fn discover() -> CoreResult<Self> {
        let data_dir = if let Ok(path) = std::env::var("MINEIA_DATA_DIR") {
            PathBuf::from(path)
        } else {
            ProjectDirs::from("com", "mineia", "MineIA")
                .ok_or(CoreError::MissingDataDirectory)?
                .data_local_dir()
                .to_path_buf()
        };

        Ok(Self {
            database_file: data_dir.join("mineia.db"),
            downloads_dir: data_dir.join("downloads"),
            versions_dir: data_dir.join("versions"),
            assets_dir: data_dir.join("assets"),
            libraries_dir: data_dir.join("libraries"),
            profiles_dir: data_dir.join("profiles"),
            logs_dir: data_dir.join("logs"),
            data_dir,
        })
    }

    pub fn ensure(&self) -> CoreResult<()> {
        self.ensure_runtime_dirs()?;
        std::fs::create_dir_all(&self.profiles_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        Ok(())
    }

    pub fn ensure_runtime_dirs(&self) -> CoreResult<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.downloads_dir)?;
        std::fs::create_dir_all(&self.versions_dir)?;
        std::fs::create_dir_all(&self.assets_dir)?;
        std::fs::create_dir_all(&self.libraries_dir)?;
        Ok(())
    }

    pub fn profile_dir(&self, profile_id: i64) -> PathBuf {
        self.profiles_dir.join(profile_id.to_string())
    }

    pub fn natives_dir(&self, version: &str) -> PathBuf {
        self.versions_dir.join(version).join("natives")
    }
}
