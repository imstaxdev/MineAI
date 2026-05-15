use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("El username debe tener entre 3 y 16 caracteres.")]
    InvalidLength,
    #[error("El username solo puede usar letras, numeros y guion bajo.")]
    InvalidCharacters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfflineProfile {
    pub username: String,
    pub uuid: Uuid,
}

pub fn create_offline_profile(username: impl AsRef<str>) -> Result<OfflineProfile, AuthError> {
    let username = username.as_ref().trim();
    validate_username(username)?;

    Ok(OfflineProfile {
        username: username.to_owned(),
        uuid: minecraft_offline_uuid(username),
    })
}

pub fn validate_username(username: &str) -> Result<(), AuthError> {
    if !(3..=16).contains(&username.len()) {
        return Err(AuthError::InvalidLength);
    }

    if !username
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(AuthError::InvalidCharacters);
    }

    Ok(())
}

pub fn minecraft_offline_uuid(username: &str) -> Uuid {
    let mut digest = Md5::digest(format!("OfflinePlayer:{username}").as_bytes());
    digest[6] = (digest[6] & 0x0f) | 0x30;
    digest[8] = (digest[8] & 0x3f) | 0x80;

    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest);
    Uuid::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_minecraft_style_usernames() {
        assert!(validate_username("MineIA_01").is_ok());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("name with spaces").is_err());
    }

    #[test]
    fn offline_uuid_is_stable() {
        assert_eq!(
            minecraft_offline_uuid("Player").to_string(),
            minecraft_offline_uuid("Player").to_string()
        );
        assert_ne!(
            minecraft_offline_uuid("Player").to_string(),
            minecraft_offline_uuid("OtherPlayer").to_string()
        );
    }
}
