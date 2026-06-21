use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Debug, Error)]
pub enum GitError {
    #[error("not a git repository: {0}")]
    NotGitRepo(String),

    #[error("git command failed: {command}: {stderr}")]
    CommandFailed { command: String, stderr: String },

    #[error("failed to read git output: {0}")]
    OutputRead(String),
}

#[async_trait]
pub trait GitGateway: Send + Sync {
    async fn branch(&self) -> Result<String, GitError>;
    async fn head_commit(&self) -> Result<String, GitError>;
    async fn status_porcelain(&self) -> Result<Vec<(String, String)>, GitError>;
    async fn log(&self, max_count: u8) -> Result<Vec<CommitSummary>, GitError>;
}

#[derive(Debug, Clone)]
pub struct ProductionGitAdapter {
    repo_root: PathBuf,
}

impl ProductionGitAdapter {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    async fn run(&self, args: &[&str]) -> Result<std::process::Output, GitError> {
        let output = Command::new("git")
            .current_dir(&self.repo_root)
            .args(args)
            .output()
            .await
            .map_err(|e| GitError::OutputRead(e.to_string()))?;

        Ok(output)
    }
}

#[async_trait]
impl GitGateway for ProductionGitAdapter {
    async fn branch(&self) -> Result<String, GitError> {
        let output = self.run(&["branch", "--show-current"]).await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(GitError::CommandFailed {
                command: "git branch --show-current".to_string(),
                stderr,
            });
        }
        let branch = String::from_utf8(output.stdout)
            .map_err(|e| GitError::OutputRead(e.to_string()))?
            .trim()
            .to_string();
        Ok(branch)
    }

    async fn head_commit(&self) -> Result<String, GitError> {
        let output = self.run(&["rev-parse", "HEAD"]).await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(GitError::CommandFailed {
                command: "git rev-parse HEAD".to_string(),
                stderr,
            });
        }
        let sha = String::from_utf8(output.stdout)
            .map_err(|e| GitError::OutputRead(e.to_string()))?
            .trim()
            .to_string();
        Ok(sha)
    }

    async fn status_porcelain(&self) -> Result<Vec<(String, String)>, GitError> {
        let output = self.run(&["status", "--porcelain"]).await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(GitError::CommandFailed {
                command: "git status --porcelain".to_string(),
                stderr,
            });
        }
        let stdout =
            String::from_utf8(output.stdout).map_err(|e| GitError::OutputRead(e.to_string()))?;
        let mut files = Vec::new();
        for line in stdout.lines() {
            if line.len() >= 3 {
                let status = &line[0..2];
                let path = line[3..].to_string();
                files.push((status.to_string(), path));
            }
        }
        Ok(files)
    }

    async fn log(&self, max_count: u8) -> Result<Vec<CommitSummary>, GitError> {
        let output = self
            .run(&[
                "log",
                "--max-count",
                &max_count.to_string(),
                "--format=%H%n%s%n%an%n%aI%n---END---",
            ])
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(GitError::CommandFailed {
                command: format!("git log --max-count={}", max_count),
                stderr,
            });
        }
        let stdout =
            String::from_utf8(output.stdout).map_err(|e| GitError::OutputRead(e.to_string()))?;
        let mut commits = Vec::new();
        for block in stdout.split("---END---") {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let mut lines = block.lines();
            let sha = lines.next().unwrap_or("").to_string();
            let message = lines.next().unwrap_or("").to_string();
            let author = lines.next().unwrap_or("").to_string();
            let timestamp = lines.next().unwrap_or("").to_string();
            let short_sha = sha.get(0..7).unwrap_or(sha.as_str()).to_string();
            commits.push(CommitSummary {
                sha,
                short_sha,
                message,
                author,
                timestamp,
            });
        }
        Ok(commits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    pub struct FakeGitAdapter {
        pub branch: String,
        pub head_commit: String,
        pub status: Vec<(String, String)>,
        pub log: Vec<CommitSummary>,
        pub fail: bool,
    }

    #[async_trait]
    impl GitGateway for FakeGitAdapter {
        async fn branch(&self) -> Result<String, GitError> {
            if self.fail {
                return Err(GitError::NotGitRepo("fake".to_string()));
            }
            Ok(self.branch.clone())
        }

        async fn head_commit(&self) -> Result<String, GitError> {
            if self.fail {
                return Err(GitError::NotGitRepo("fake".to_string()));
            }
            Ok(self.head_commit.clone())
        }

        async fn status_porcelain(&self) -> Result<Vec<(String, String)>, GitError> {
            if self.fail {
                return Err(GitError::NotGitRepo("fake".to_string()));
            }
            Ok(self.status.clone())
        }

        async fn log(&self, _max_count: u8) -> Result<Vec<CommitSummary>, GitError> {
            if self.fail {
                return Err(GitError::NotGitRepo("fake".to_string()));
            }
            Ok(self.log.clone())
        }
    }

    #[test]
    fn fake_adapter_returns_branch() {
        let adapter = FakeGitAdapter {
            branch: "main".to_string(),
            head_commit: "abc123".to_string(),
            status: vec![],
            log: vec![],
            fail: false,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let branch = rt.block_on(adapter.branch()).unwrap();
        assert_eq!(branch, "main");
    }

    #[test]
    fn fake_adapter_returns_error_when_fail() {
        let adapter = FakeGitAdapter {
            branch: "main".to_string(),
            head_commit: "abc123".to_string(),
            status: vec![],
            log: vec![],
            fail: true,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(adapter.branch());
        assert!(result.is_err());
    }
}
