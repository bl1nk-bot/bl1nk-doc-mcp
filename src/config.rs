use std::path::PathBuf;

use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("repository root does not exist: {path}")]
    RepoRootMissing { path: String },

    #[error("invalid repository root: {reason}")]
    InvalidRepoRoot { reason: String },
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub repo_root: PathBuf,
}

impl ServerConfig {
    pub fn from_env_or_cwd() -> Result<Self> {
        let repo_root = std::env::args()
            .nth(1)
            .or_else(|| std::env::var("BL1NK_REPO_ROOT").ok())
            .unwrap_or_else(|| std::env::current_dir().unwrap().to_string_lossy().into());

        Self::new(repo_root)
    }

    pub fn new(repo_root: impl Into<String>) -> Result<Self> {
        let path = PathBuf::from(repo_root.into());
        let canonical = std::fs::canonicalize(&path)
            .with_context(|| format!("failed to resolve repo root path: {}", path.display()))?;

        if !canonical.exists() {
            anyhow::bail!(ServerError::RepoRootMissing {
                path: canonical.to_string_lossy().to_string()
            });
        }

        Ok(Self {
            repo_root: canonical,
        })
    }
}
