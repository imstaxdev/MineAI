use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::CoreResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashes {
    pub blake3: String,
    pub sha256: String,
}

pub fn hash_file(path: impl AsRef<Path>) -> CoreResult<FileHashes> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha256_hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        blake3_hasher.update(&buffer[..read]);
        sha256_hasher.update(&buffer[..read]);
    }

    Ok(FileHashes {
        blake3: blake3_hasher.finalize().to_hex().to_string(),
        sha256: hex::encode(sha256_hasher.finalize()),
    })
}
