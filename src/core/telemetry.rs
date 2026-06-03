//! anorak-rtk: telemetry network egress is hard-disabled at build time.
//!
//! Upstream rtk includes an opt-in usage-ping that POSTs aggregated stats to
//! a remote server. The Yui-Qi-Tang #640 C-1 fix added a runtime kill-switch
//! requiring `RTK_TELEMETRY_FORCE_SEND=1` per process. This fork goes further:
//! the network call site, the payload-assembly helpers, and the `ureq` HTTP
//! client are removed at compile time. `maybe_ping()` is a no-op.
//!
//! What remains is the device-salt scaffolding used by `rtk telemetry status`
//! / `rtk telemetry erase` for local reporting. No bytes leave the host.

use super::constants::RTK_DATA_DIR;
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::sync::OnceLock;

static CACHED_SALT: OnceLock<String> = OnceLock::new();

/// No-op. anorak-rtk never pings any remote server.
pub fn maybe_ping() {}

pub fn generate_device_hash() -> String {
    let salt = get_or_create_salt();
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_or_create_salt() -> String {
    CACHED_SALT
        .get_or_init(|| {
            let salt_path = salt_file_path();

            if let Ok(contents) = std::fs::read_to_string(&salt_path) {
                let trimmed = contents.trim().to_string();
                if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                    return trimmed;
                }
            }

            let salt = random_salt();
            if let Some(parent) = salt_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut f) = std::fs::File::create(&salt_path) {
                let _ = f.write_all(salt.as_bytes());
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &salt_path,
                        std::fs::Permissions::from_mode(0o600),
                    );
                }
            }
            salt
        })
        .clone()
}

fn random_salt() -> String {
    let mut buf = [0u8; 32];
    if getrandom::fill(&mut buf).is_err() {
        let fallback = format!("{:?}:{}", std::time::SystemTime::now(), std::process::id());
        let mut hasher = Sha256::new();
        hasher.update(fallback.as_bytes());
        return format!("{:x}", hasher.finalize());
    }
    buf.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}

pub fn salt_file_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("rtk")
        .join(".device_salt")
}

pub fn telemetry_marker_path() -> PathBuf {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(RTK_DATA_DIR);
    let _ = std::fs::create_dir_all(&data_dir);
    data_dir.join(".telemetry_last_ping")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_hash_is_stable() {
        let h1 = generate_device_hash();
        let h2 = generate_device_hash();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_device_hash_is_valid_hex() {
        let hash = generate_device_hash();
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_salt_is_persisted() {
        let s1 = get_or_create_salt();
        let s2 = get_or_create_salt();
        assert_eq!(s1, s2);
        assert_eq!(s1.len(), 64);
        assert!(s1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_random_salt_uniqueness() {
        let s1 = random_salt();
        let s2 = random_salt();
        assert_ne!(s1, s2);
        assert_eq!(s1.len(), 64);
        assert_eq!(s2.len(), 64);
    }

    #[test]
    fn test_salt_file_path_is_in_rtk_dir() {
        let path = salt_file_path();
        assert!(path.to_string_lossy().contains("rtk"));
        assert!(path.to_string_lossy().contains(".device_salt"));
    }

    #[test]
    fn test_marker_path_exists() {
        let path = telemetry_marker_path();
        assert!(path.to_string_lossy().contains("rtk"));
    }

    #[test]
    fn test_maybe_ping_is_a_noop() {
        // Will deadlock or panic if it ever reaches the original network code.
        maybe_ping();
    }
}
