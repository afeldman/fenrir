//! Path canonicalization and validation

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Path-related errors
#[derive(Error, Debug)]
pub enum PathError {
    #[error("Path does not exist: {0:?}")]
    NotFound(PathBuf),
    
    #[error("Path is not a directory: {0:?}")]
    NotADirectory(PathBuf),
    
    #[error("Path is not a file: {0:?}")]
    NotAFile(PathBuf),
    
    #[error("Path traversal attempt detected: {0:?}")]
    PathTraversal(PathBuf),
    
    #[error("Invalid path: {0:?}")]
    InvalidPath(PathBuf),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Canonicalize a path, resolving symlinks and checking for traversal attempts
pub fn canonicalize_path(path: &Path) -> Result<PathBuf, PathError> {
    // Check for path traversal attempts
    let components: Vec<_> = path.components().collect();
    let mut depth = 0;
    
    for component in &components {
        match component {
            std::path::Component::ParentDir => {
                if depth == 0 {
                    return Err(PathError::PathTraversal(path.to_path_buf()));
                }
                depth -= 1;
            }
            std::path::Component::Normal(_) => {
                depth += 1;
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                // Reset depth at root
                depth = 0;
            }
            std::path::Component::CurDir => {
                // Current directory doesn't change depth
            }
        }
    }
    
    // Get absolute path
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    
    // Canonicalize (resolve symlinks)
    let canonical = absolute_path.canonicalize()?;
    
    Ok(canonical)
}

/// Validate that a path exists and is a directory
pub fn validate_directory(path: &Path) -> Result<PathBuf, PathError> {
    let canonical = canonicalize_path(path)?;
    
    if !canonical.exists() {
        return Err(PathError::NotFound(canonical));
    }
    
    if !canonical.is_dir() {
        return Err(PathError::NotADirectory(canonical));
    }
    
    Ok(canonical)
}

/// Validate that a path exists and is a file
pub fn validate_file(path: &Path) -> Result<PathBuf, PathError> {
    let canonical = canonicalize_path(path)?;
    
    if !canonical.exists() {
        return Err(PathError::NotFound(canonical));
    }
    
    if !canonical.is_file() {
        return Err(PathError::NotAFile(canonical));
    }
    
    Ok(canonical)
}

/// Check if a path is within a base directory
pub fn is_within_directory(path: &Path, base_dir: &Path) -> bool {
    let canonical_path = match canonicalize_path(path) {
        Ok(p) => p,
        Err(_) => return false,
    };
    
    let canonical_base = match canonicalize_path(base_dir) {
        Ok(b) => b,
        Err(_) => return false,
    };
    
    canonical_path.starts_with(&canonical_base)
}
