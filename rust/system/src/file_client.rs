//! File client for component artifact management.
//!
//! Provides a simple interface for system components to read and write
//! files within their designated storage directory. Used for persisting
//! component state, caching data, and managing downloaded artifacts.

use std::path::{Path, PathBuf};
use tokio::fs;
use thiserror::Error;

/// Errors that can occur during file operations.
#[derive(Debug, Error)]
pub enum FileClientError {
    /// The target path is outside the allowed base directory.
    #[error("path traversal denied: {0} is outside the component directory")]
    PathTraversal(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A sandboxed file client that restricts operations to a base directory.
#[derive(Debug, Clone)]
pub struct FileClient {
    base_dir: PathBuf,
}

impl FileClient {
    /// Create a new file client rooted at the given directory.
    ///
    /// All paths passed to read/write methods are resolved relative to this base.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Resolve a relative path against the base directory, rejecting traversal.
    fn resolve(&self, relative: &str) -> Result<PathBuf, FileClientError> {
        let resolved = self.base_dir.join(relative);
        let canonical_base = self.base_dir.canonicalize().unwrap_or_else(|_| self.base_dir.clone());
        let canonical_target = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());

        if !canonical_target.starts_with(&canonical_base) {
            return Err(FileClientError::PathTraversal(relative.to_string()));
        }
        Ok(resolved)
    }

    /// Read a file as bytes.
    pub async fn read(&self, path: &str) -> Result<Vec<u8>, FileClientError> {
        let resolved = self.resolve(path)?;
        Ok(fs::read(&resolved).await?)
    }

    /// Read a file as a UTF-8 string.
    pub async fn read_string(&self, path: &str) -> Result<String, FileClientError> {
        let resolved = self.resolve(path)?;
        Ok(fs::read_to_string(&resolved).await?)
    }

    /// Write bytes to a file, creating parent directories as needed.
    pub async fn write(&self, path: &str, data: &[u8]) -> Result<(), FileClientError> {
        let resolved = self.resolve(path)?;
        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(fs::write(&resolved, data).await?)
    }

    /// Write a string to a file, creating parent directories as needed.
    pub async fn write_string(&self, path: &str, content: &str) -> Result<(), FileClientError> {
        self.write(path, content.as_bytes()).await
    }

    /// Check if a file exists.
    pub async fn exists(&self, path: &str) -> Result<bool, FileClientError> {
        let resolved = self.resolve(path)?;
        Ok(resolved.exists())
    }

    /// Delete a file.
    pub async fn remove(&self, path: &str) -> Result<(), FileClientError> {
        let resolved = self.resolve(path)?;
        Ok(fs::remove_file(&resolved).await?)
    }

    /// Get the base directory path.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}
