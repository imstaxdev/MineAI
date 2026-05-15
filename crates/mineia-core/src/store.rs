use std::path::PathBuf;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::{CoreResult, CreateProfileRequest, Profile, ProfileSettings};

#[derive(Debug, Clone)]
pub struct MineiaStore {
    db_path: PathBuf,
}

impl MineiaStore {
    pub fn open(db_path: PathBuf) -> CoreResult<Self> {
        Ok(Self { db_path })
    }

    pub fn migrate(&self) -> CoreResult<()> {
        let connection = self.connection()?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                offline_uuid TEXT NOT NULL UNIQUE,
                minecraft_version TEXT NOT NULL,
                ram_mb INTEGER NOT NULL,
                java_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_profile(&self, request: CreateProfileRequest) -> CoreResult<Profile> {
        let offline = mineia_auth::create_offline_profile(request.username)?;
        let now = Utc::now().to_rfc3339();
        let connection = self.connection()?;

        connection.execute(
            r#"
            INSERT INTO profiles
                (username, offline_uuid, minecraft_version, ram_mb, java_path, created_at, updated_at)
            VALUES
                (?1, ?2, ?3, ?4, NULL, ?5, ?5)
            "#,
            params![offline.username, offline.uuid.to_string(), "26.1.2", 2048_u32, now],
        )?;

        self.get_profile(connection.last_insert_rowid())
    }

    pub fn list_profiles(&self) -> CoreResult<Vec<Profile>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            r#"
            SELECT id, username, offline_uuid, minecraft_version, ram_mb, java_path, created_at, updated_at
            FROM profiles
            ORDER BY updated_at DESC, id DESC
            "#,
        )?;

        let profiles = statement
            .query_map([], Self::row_to_profile)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }

    pub fn get_profile(&self, profile_id: i64) -> CoreResult<Profile> {
        let connection = self.connection()?;
        connection
            .query_row(
                r#"
                SELECT id, username, offline_uuid, minecraft_version, ram_mb, java_path, created_at, updated_at
                FROM profiles
                WHERE id = ?1
                "#,
                params![profile_id],
                Self::row_to_profile,
            )
            .map_err(Into::into)
    }

    pub fn update_profile_settings(
        &self,
        profile_id: i64,
        settings: ProfileSettings,
    ) -> CoreResult<Profile> {
        let current = self.get_profile(profile_id)?;
        let minecraft_version = settings
            .minecraft_version
            .unwrap_or(current.minecraft_version);
        let ram_mb = settings.ram_mb.unwrap_or(current.ram_mb).clamp(512, 32_768);
        let java_path = settings
            .java_path
            .or(current.java_path)
            .map(|path| path.to_string_lossy().to_string());
        let now = Utc::now().to_rfc3339();

        let connection = self.connection()?;
        connection.execute(
            r#"
            UPDATE profiles
            SET minecraft_version = ?1,
                ram_mb = ?2,
                java_path = ?3,
                updated_at = ?4
            WHERE id = ?5
            "#,
            params![minecraft_version, ram_mb, java_path, now, profile_id],
        )?;

        self.get_profile(profile_id)
    }

    pub fn profile_exists(&self, profile_id: i64) -> CoreResult<bool> {
        let connection = self.connection()?;
        let found = connection
            .query_row(
                "SELECT 1 FROM profiles WHERE id = ?1",
                params![profile_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        Ok(found)
    }

    fn connection(&self) -> CoreResult<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn row_to_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<Profile> {
        let java_path: Option<String> = row.get(5)?;
        Ok(Profile {
            id: row.get(0)?,
            username: row.get(1)?,
            offline_uuid: row.get(2)?,
            minecraft_version: row.get(3)?,
            ram_mb: row.get(4)?,
            java_path: java_path.map(PathBuf::from),
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}
