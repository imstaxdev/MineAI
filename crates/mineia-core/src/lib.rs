mod hashes;
mod paths;
mod profiles;
mod store;

pub use hashes::{hash_file, FileHashes};
pub use paths::MineiaPaths;
pub use profiles::{CreateProfileRequest, Profile, ProfileSettings};
pub use store::MineiaStore;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Auth(#[from] mineia_auth::AuthError),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No se pudo encontrar una carpeta de datos local para MineIA.")]
    MissingDataDirectory,
}

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone)]
pub struct MineiaContext {
    pub paths: MineiaPaths,
    pub store: MineiaStore,
}

impl MineiaContext {
    pub fn open_default() -> CoreResult<Self> {
        let paths = MineiaPaths::discover()?;
        paths.ensure()?;

        let store = MineiaStore::open(paths.database_file.clone())?;
        store.migrate()?;

        Ok(Self { paths, store })
    }
}
