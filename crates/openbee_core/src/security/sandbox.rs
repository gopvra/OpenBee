//! Filesystem sandbox — ensures the engine can ONLY access user-approved directories.
//!
//! Every file operation in OpenBee (asset loading, save files, mods, screenshots,
//! replays, configs) goes through [`SandboxedFs`] which validates that the resolved
//! path stays within the allowed directory set. This prevents path traversal attacks
//! and guarantees users' personal files are never touched.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use thiserror::Error;
use tracing::warn;

/// Errors produced by the sandbox layer.
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Access denied: path '{path}' is outside all allowed directories")]
    AccessDenied { path: String },

    #[error("Path traversal detected: '{path}' contains forbidden components")]
    PathTraversal { path: String },

    #[error("Symlink escape: '{path}' resolves outside the sandbox")]
    SymlinkEscape { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Sandbox not initialized — call add_allowed_directory first")]
    NotInitialized,
}

pub type SandboxResult<T> = Result<T, SandboxError>;

/// Permission levels for directory access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Read-only access (asset loading).
    ReadOnly,
    /// Read and write access (saves, screenshots, configs).
    ReadWrite,
}

/// A single allowed directory entry.
#[derive(Debug, Clone)]
struct AllowedDir {
    /// Canonical (absolute, resolved) path of the allowed directory.
    canonical: PathBuf,
    /// What the engine is allowed to do here.
    permission: Permission,
    /// Human-readable label for logging.
    label: String,
}

/// Thread-safe filesystem sandbox.
///
/// # Usage
/// ```ignore
/// let sandbox = SandboxedFs::new();
/// sandbox.add_allowed_directory("./assets", Permission::ReadOnly, "Game Assets")?;
/// sandbox.add_allowed_directory("./saves",  Permission::ReadWrite, "Save Files")?;
///
/// // These succeed:
/// sandbox.validate_read("./assets/CLAW.REZ")?;
/// sandbox.validate_write("./saves/slot1.json")?;
///
/// // These are DENIED:
/// sandbox.validate_read("/etc/passwd")?;           // outside sandbox
/// sandbox.validate_read("./assets/../../secret")?;  // path traversal
/// ```
pub struct SandboxedFs {
    allowed: Arc<RwLock<Vec<AllowedDir>>>,
    /// Maximum allowed path depth (defence in depth).
    max_path_depth: usize,
    /// Maximum allowed filename length.
    max_filename_len: usize,
    /// Blocked filename patterns (e.g., dotfiles, system files).
    blocked_patterns: Vec<String>,
}

impl SandboxedFs {
    /// Create a new sandbox with no allowed directories.
    pub fn new() -> Self {
        Self {
            allowed: Arc::new(RwLock::new(Vec::new())),
            max_path_depth: 32,
            max_filename_len: 255,
            blocked_patterns: vec![
                ".env".into(),
                ".git".into(),
                ".ssh".into(),
                "id_rsa".into(),
                "shadow".into(),
                "passwd".into(),
                "credentials".into(),
                "secret".into(),
                ".aws".into(),
                "private_key".into(),
            ],
        }
    }

    /// Register a directory the engine is allowed to access.
    ///
    /// The directory is canonicalized (symlinks resolved, `..` removed) so that
    /// later checks compare resolved absolute paths.
    pub fn add_allowed_directory(
        &self,
        path: impl AsRef<Path>,
        permission: Permission,
        label: &str,
    ) -> SandboxResult<()> {
        let canonical = std::fs::canonicalize(path.as_ref()).map_err(|e| {
            warn!(
                "Cannot canonicalize allowed directory '{}': {}",
                path.as_ref().display(),
                e
            );
            SandboxError::Io(e)
        })?;

        let mut allowed = self.allowed.write().unwrap();
        // Avoid duplicates.
        if !allowed.iter().any(|d| d.canonical == canonical) {
            tracing::info!(
                "Sandbox: allowing {} access to '{}' ({})",
                match permission {
                    Permission::ReadOnly => "read",
                    Permission::ReadWrite => "read/write",
                },
                canonical.display(),
                label
            );
            allowed.push(AllowedDir {
                canonical,
                permission,
                label: label.to_string(),
            });
        }
        Ok(())
    }

    /// Remove a previously allowed directory.
    pub fn remove_allowed_directory(&self, path: impl AsRef<Path>) -> SandboxResult<()> {
        let canonical = std::fs::canonicalize(path.as_ref())?;
        let mut allowed = self.allowed.write().unwrap();
        allowed.retain(|d| d.canonical != canonical);
        Ok(())
    }

    /// List all currently allowed directories.
    pub fn list_allowed(&self) -> Vec<(String, Permission, String)> {
        let allowed = self.allowed.read().unwrap();
        allowed
            .iter()
            .map(|d| (d.canonical.display().to_string(), d.permission, d.label.clone()))
            .collect()
    }

    // ---- Validation ----

    /// Validate that a read operation on `path` is permitted.
    pub fn validate_read(&self, path: impl AsRef<Path>) -> SandboxResult<PathBuf> {
        let resolved = self.resolve_and_check(path.as_ref())?;
        let allowed = self.allowed.read().unwrap();
        for dir in allowed.iter() {
            if resolved.starts_with(&dir.canonical) {
                return Ok(resolved);
            }
        }
        Err(SandboxError::AccessDenied {
            path: path.as_ref().display().to_string(),
        })
    }

    /// Validate that a write operation on `path` is permitted.
    pub fn validate_write(&self, path: impl AsRef<Path>) -> SandboxResult<PathBuf> {
        let resolved = self.resolve_and_check(path.as_ref())?;
        let allowed = self.allowed.read().unwrap();
        for dir in allowed.iter() {
            if resolved.starts_with(&dir.canonical) && dir.permission == Permission::ReadWrite {
                return Ok(resolved);
            }
        }
        Err(SandboxError::AccessDenied {
            path: path.as_ref().display().to_string(),
        })
    }

    /// Read a file, but only if the sandbox allows it.
    pub fn read(&self, path: impl AsRef<Path>) -> SandboxResult<Vec<u8>> {
        let resolved = self.validate_read(path)?;
        Ok(std::fs::read(resolved)?)
    }

    /// Read a file as UTF-8 text, sandbox-checked.
    pub fn read_to_string(&self, path: impl AsRef<Path>) -> SandboxResult<String> {
        let resolved = self.validate_read(path)?;
        Ok(std::fs::read_to_string(resolved)?)
    }

    /// Write data to a file, sandbox-checked.
    pub fn write(&self, path: impl AsRef<Path>, data: &[u8]) -> SandboxResult<()> {
        let resolved = self.validate_write(&path)?;
        // Create parent directories if needed (still within sandbox).
        if let Some(parent) = resolved.parent() {
            if !parent.exists() {
                self.validate_write(parent)?;
                std::fs::create_dir_all(parent)?;
            }
        }
        Ok(std::fs::write(resolved, data)?)
    }

    /// Write a string to a file, sandbox-checked.
    pub fn write_string(&self, path: impl AsRef<Path>, data: &str) -> SandboxResult<()> {
        self.write(path, data.as_bytes())
    }

    /// List directory contents, sandbox-checked.
    pub fn read_dir(&self, path: impl AsRef<Path>) -> SandboxResult<Vec<PathBuf>> {
        let resolved = self.validate_read(path)?;
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(resolved)? {
            let entry = entry?;
            entries.push(entry.path());
        }
        Ok(entries)
    }

    /// Check if a path exists, sandbox-checked.
    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.validate_read(path).map(|p| p.exists()).unwrap_or(false)
    }

    // ---- Internal ----

    /// Resolve a path and perform all security checks.
    fn resolve_and_check(&self, path: &Path) -> SandboxResult<PathBuf> {
        // 1. Check for obviously malicious path components.
        let path_str = path.to_string_lossy();

        // Reject null bytes.
        if path_str.contains('\0') {
            return Err(SandboxError::PathTraversal {
                path: path_str.into_owned(),
            });
        }

        // 2. Check path depth.
        let component_count = path.components().count();
        if component_count > self.max_path_depth {
            return Err(SandboxError::PathTraversal {
                path: format!("Path too deep ({} components, max {})", component_count, self.max_path_depth),
            });
        }

        // 3. Check filename length.
        if let Some(name) = path.file_name() {
            if name.len() > self.max_filename_len {
                return Err(SandboxError::PathTraversal {
                    path: format!("Filename too long ({} chars)", name.len()),
                });
            }
        }

        // 4. Check for blocked patterns.
        for component in path.components() {
            let comp_str = component.as_os_str().to_string_lossy();
            for pattern in &self.blocked_patterns {
                if comp_str.eq_ignore_ascii_case(pattern) {
                    return Err(SandboxError::PathTraversal {
                        path: format!("Blocked pattern '{}' in path", pattern),
                    });
                }
            }
        }

        // 5. Canonicalize to resolve symlinks and `..`.
        // If the file doesn't exist yet (writes), canonicalize the parent.
        let canonical = if path.exists() {
            std::fs::canonicalize(path)?
        } else {
            // For new files, canonicalize the parent directory.
            let parent = path.parent().unwrap_or(Path::new("."));
            if parent.exists() {
                let canonical_parent = std::fs::canonicalize(parent)?;
                let file_name = path
                    .file_name()
                    .ok_or_else(|| SandboxError::PathTraversal {
                        path: path_str.into_owned(),
                    })?;
                canonical_parent.join(file_name)
            } else {
                return Err(SandboxError::AccessDenied {
                    path: path_str.into_owned(),
                });
            }
        };

        Ok(canonical)
    }
}

impl Default for SandboxedFs {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Convenience: global sandbox instance ----

use std::sync::OnceLock;

static GLOBAL_SANDBOX: OnceLock<SandboxedFs> = OnceLock::new();

/// Get or initialize the global filesystem sandbox.
pub fn global_sandbox() -> &'static SandboxedFs {
    GLOBAL_SANDBOX.get_or_init(SandboxedFs::new)
}

/// Initialize the global sandbox with standard game directories.
///
/// Call this once at startup before any file operations.
pub fn init_sandbox(
    game_data_dir: &Path,
    save_dir: &Path,
    mods_dir: &Path,
    screenshots_dir: &Path,
    config_dir: &Path,
) -> SandboxResult<()> {
    let sandbox = global_sandbox();
    sandbox.add_allowed_directory(game_data_dir, Permission::ReadOnly, "Game Data (CLAW.REZ)")?;
    sandbox.add_allowed_directory(save_dir, Permission::ReadWrite, "Save Files")?;
    sandbox.add_allowed_directory(mods_dir, Permission::ReadOnly, "Mods")?;
    sandbox.add_allowed_directory(screenshots_dir, Permission::ReadWrite, "Screenshots")?;
    sandbox.add_allowed_directory(config_dir, Permission::ReadWrite, "Configuration")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sandbox_read_allowed() {
        let dir = std::env::temp_dir().join("openbee_sandbox_test_read");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("test.txt"), b"hello").unwrap();

        let sandbox = SandboxedFs::new();
        sandbox
            .add_allowed_directory(&dir, Permission::ReadOnly, "test")
            .unwrap();

        assert!(sandbox.validate_read(dir.join("test.txt")).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sandbox_read_denied() {
        let sandbox = SandboxedFs::new();
        // No directories allowed — everything is denied.
        assert!(sandbox.validate_read("/etc/passwd").is_err());
        assert!(sandbox.validate_read("/tmp/anything").is_err());
    }

    #[test]
    fn test_sandbox_write_denied_on_readonly() {
        let dir = std::env::temp_dir().join("openbee_sandbox_test_ro");
        let _ = fs::create_dir_all(&dir);

        let sandbox = SandboxedFs::new();
        sandbox
            .add_allowed_directory(&dir, Permission::ReadOnly, "test")
            .unwrap();

        // Read OK, write denied.
        assert!(sandbox.validate_read(&dir).is_ok());
        assert!(sandbox.validate_write(dir.join("file.txt")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sandbox_blocks_traversal_patterns() {
        let sandbox = SandboxedFs::new();
        assert!(sandbox.validate_read(Path::new("/some/.env/file")).is_err());
        assert!(sandbox.validate_read(Path::new("/path/to/.ssh/key")).is_err());
        assert!(sandbox.validate_read(Path::new("/path/to/.git/config")).is_err());
    }

    #[test]
    fn test_sandbox_null_byte_rejection() {
        let sandbox = SandboxedFs::new();
        let bad = PathBuf::from("/tmp/test\0evil");
        assert!(sandbox.resolve_and_check(&bad).is_err());
    }

    #[test]
    fn test_sandbox_write_allowed() {
        let dir = std::env::temp_dir().join("openbee_sandbox_test_rw");
        let _ = fs::create_dir_all(&dir);

        let sandbox = SandboxedFs::new();
        sandbox
            .add_allowed_directory(&dir, Permission::ReadWrite, "test")
            .unwrap();

        let file_path = dir.join("output.txt");
        assert!(sandbox.write(&file_path, b"data").is_ok());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "data");
        let _ = fs::remove_dir_all(&dir);
    }
}
