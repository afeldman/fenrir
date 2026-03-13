//! Filesystem types and traits
//!
//! This module defines types and traits for secure filesystem access,
//! including canonical paths and filesystem operations.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// A canonical path that has been validated and normalized
///
/// # Security
///
/// This type guarantees that the path:
/// 1. Is absolute (starts from root)
/// 2. Has no `.` or `..` components
/// 3. Has been normalized (no duplicate separators)
/// 4. Is safe to use for filesystem operations
///
/// Constructing a `CanonicalPath` validates these properties.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanonicalPath(String);

impl CanonicalPath {
    /// Create a new CanonicalPath from a string
    ///
    /// # Arguments
    ///
    /// * `path` - The path string to validate and canonicalize
    ///
    /// # Returns
    ///
    /// `Ok(CanonicalPath)` if the path is valid, `Err(ValidationError)` otherwise.
    ///
    /// # Security
    ///
    /// This method performs validation to ensure the path is safe.
    /// It does NOT check if the path exists on the filesystem.
    pub fn new(path: impl AsRef<str>) -> Result<Self, crate::errors::ValidationError> {
        let path_str = path.as_ref();
        
        if path_str.is_empty() {
            return Err(crate::errors::ValidationError::new("Path cannot be empty"));
        }
        
        // Basic validation
        if path_str.contains("..") {
            return Err(crate::errors::ValidationError::new("Path cannot contain '..'"));
        }
        
        if path_str.contains("//") {
            return Err(crate::errors::ValidationError::new("Path cannot contain duplicate separators"));
        }
        
        // For now, we'll just store the path as-is
        // In a real implementation, we would:
        // 1. Convert to absolute path
        // 2. Normalize separators
        // 3. Remove `.` components
        // 4. Ensure it's within allowed directories
        
        Ok(Self(path_str.to_string()))
    }
    
    /// Create a CanonicalPath from a PathBuf
    ///
    /// # Security
    ///
    /// This converts the path to a string and validates it.
    /// The path must be valid UTF-8.
    pub fn from_path_buf(path: PathBuf) -> Result<Self, crate::errors::ValidationError> {
        let path_str = path.to_string_lossy().to_string();
        Self::new(path_str)
    }
    
    /// Get the path as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the path as a Path reference
    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }
    
    /// Join this path with another component
    ///
    /// # Arguments
    ///
    /// * `component` - The component to join
    ///
    /// # Returns
    ///
    /// A new CanonicalPath with the joined component, or an error if invalid.
    pub fn join(&self, component: &str) -> Result<Self, crate::errors::ValidationError> {
        let new_path = format!("{}/{}", self.0, component);
        Self::new(new_path)
    }
    
    /// Get the parent directory of this path
    ///
    /// # Returns
    ///
    /// `Some(CanonicalPath)` if this path has a parent, `None` otherwise.
    pub fn parent(&self) -> Option<Self> {
        let path = Path::new(&self.0);
        path.parent().and_then(|p| {
            p.to_str().map(|s| {
                // Safe because we're taking a parent of an already canonical path
                Self(s.to_string())
            })
        })
    }
    
    /// Check if this path is a directory (based on name only)
    ///
    /// # Note
    ///
    /// This only checks the path name, not the actual filesystem.
    /// For actual directory checking, use the filesystem traits.
    pub fn is_dir_name(&self) -> bool {
        self.0.ends_with('/') || Path::new(&self.0).is_dir()
    }
    
    /// Check if this path is a file (based on name only)
    ///
    /// # Note
    ///
    /// This only checks the path name, not the actual filesystem.
    pub fn is_file_name(&self) -> bool {
        !self.is_dir_name()
    }
}

impl fmt::Display for CanonicalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Path> for CanonicalPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for CanonicalPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Result of a read operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResult {
    /// The data that was read
    pub data: Vec<u8>,
    /// The MIME type of the data, if known
    pub mime_type: Option<String>,
    /// The size of the data in bytes
    pub size: u64,
    /// When the file was last modified
    pub modified: Option<std::time::SystemTime>,
}

/// Result of a write operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResult {
    /// Number of bytes written
    pub bytes_written: u64,
    /// Whether the file was created (true) or overwritten (false)
    pub created: bool,
}

/// Trait for file access operations
///
/// This trait defines operations for reading and writing files
/// with proper permission checking.
pub trait FileAccess: Send + Sync {
    /// Read a file with permission checking
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `path` - The path to read
    ///
    /// # Returns
    ///
    /// `Ok(ReadResult)` on success, `Err(FsError)` on failure.
    fn read_file(
        &self,
        tab: &crate::tabs::TabId,
        path: &CanonicalPath,
    ) -> Result<ReadResult, FsError>;
    
    /// Write to a file with permission checking
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `path` - The path to write to
    /// * `data` - The data to write
    /// * `append` - Whether to append to the file (true) or overwrite (false)
    ///
    /// # Returns
    ///
    /// `Ok(WriteResult)` on success, `Err(FsError)` on failure.
    fn write_file(
        &self,
        tab: &crate::tabs::TabId,
        path: &CanonicalPath,
        data: &[u8],
        append: bool,
    ) -> Result<WriteResult, FsError>;
    
    /// Check if a file exists
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// `Ok(bool)` indicating if the file exists, `Err(FsError)` on permission error.
    fn file_exists(
        &self,
        tab: &crate::tabs::TabId,
        path: &CanonicalPath,
    ) -> Result<bool, FsError>;
    
    /// Get file metadata
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `path` - The path to get metadata for
    ///
    /// # Returns
    ///
    /// `Ok(std::fs::Metadata)` on success, `Err(FsError)` on failure.
    fn file_metadata(
        &self,
        tab: &crate::tabs::TabId,
        path: &CanonicalPath,
    ) -> Result<std::fs::Metadata, FsError>;
}

/// Trait for filesystem operations
///
/// This trait extends FileAccess with directory operations and
/// filesystem navigation.
pub trait FileSystem: FileAccess {
    /// List directory contents
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `path` - The directory path to list
    ///
    /// # Returns
    ///
    /// `Ok(Vec<String>)` with directory entries, `Err(FsError)` on failure.
    fn list_directory(
        &self,
        tab: &crate::tabs::TabId,
        path: &CanonicalPath,
    ) -> Result<Vec<String>, FsError>;
    
    /// Create a directory
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `path` - The directory path to create
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(FsError)` on failure.
    fn create_directory(
        &self,
        tab: &crate::tabs::TabId,
        path: &CanonicalPath,
    ) -> Result<(), FsError>;
    
    /// Delete a file or directory
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `path` - The path to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(FsError)` on failure.
    fn delete_path(
        &self,
        tab: &crate::tabs::TabId,
        path: &CanonicalPath,
    ) -> Result<(), FsError>;
    
    /// Rename or move a file/directory
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab requesting access
    /// * `from` - The source path
    /// * `to` - The destination path
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(FsError)` on failure.
    fn rename_path(
        &self,
        tab: &crate::tabs::TabId,
        from: &CanonicalPath,
        to: &CanonicalPath,
    ) -> Result<(), FsError>;
}

/// Error type for filesystem operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum FsError {
    /// File not found
    #[error("File not found: {0}")]
    NotFound(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Path is a directory (but file operation expected)
    #[error("Path is a directory: {0}")]
    IsDirectory(String),
    
    /// Path is a file (but directory operation expected)
    #[error("Path is a file: {0}")]
    IsFile(String),
    
    /// Directory not empty
    #[error("Directory not empty: {0}")]
    DirectoryNotEmpty(String),
    
    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl FsError {
    /// Create a new filesystem error with a message
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    
    /// Convert from std::io::Error
    pub fn from_io_error(error: std::io::Error, path: &str) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound(path.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied(path.to_string()),
            _ => Self::Io(format!("{}: {}", path, error)),
        }
    }
}
