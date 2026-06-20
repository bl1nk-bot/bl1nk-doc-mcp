use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("repository root does not exist: {path}")]
    RepoRootMissing { path: String },

    #[error("invalid repository root: {reason}")]
    InvalidRepoRoot { reason: String },

    #[error("path traversal detected: {path}")]
    PathTraversal { path: String },

    #[error("file not found: {path}")]
    FileNotFound { path: String },
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

pub trait RepositoryPathResolver {
    fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf>;
    fn assert_inside_repo(&self, path: impl AsRef<Path>) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct SafeRepositoryFs {
    repo_root: PathBuf,
}

impl SafeRepositoryFs {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    pub fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let relative = relative.as_ref();
        let components = relative.components().collect::<Vec<_>>();
        for component in &components {
            if matches!(component, std::path::Component::ParentDir) {
                anyhow::bail!(ServerError::PathTraversal {
                    path: relative.display().to_string()
                });
            }
        }

        let target = self.repo_root.join(relative);
        let canonical = std::fs::canonicalize(&target)
            .with_context(|| format!("failed to canonicalize path: {}", target.display()))?;

        if !canonical.starts_with(&self.repo_root) {
            anyhow::bail!(ServerError::PathTraversal {
                path: target.display().to_string()
            });
        }

        Ok(canonical)
    }

    pub fn exists(&self, relative: impl AsRef<Path>) -> Result<bool> {
        let path = self.resolve(relative)?;
        Ok(path.exists())
    }

    pub fn read(&self, relative: impl AsRef<Path>) -> Result<String> {
        let path = self.resolve(relative)?;
        if !path.exists() {
            anyhow::bail!(ServerError::FileNotFound {
                path: path.display().to_string()
            });
        }
        std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read file: {}", path.display()))
    }

    pub fn write(&self, relative: impl AsRef<Path>, contents: impl Into<String>) -> Result<()> {
        let path = self.resolve(relative)?;
        std::fs::write(&path, contents.into())
            .with_context(|| format!("failed to write file: {}", path.display()))
    }

    pub fn append(&self, relative: impl AsRef<Path>, contents: impl Into<String>) -> Result<u64> {
        let path = self.resolve(relative)?;
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("failed to open file for append: {}", path.display()))?;
        use std::io::Write;
        let size = file.write(contents.into().as_bytes())?;
        Ok(size as u64)
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }
}
