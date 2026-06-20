use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error as McpError, RequestHandlerExtra, Result as McpResult, ToolHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::adapters::filesystem::SafeRepositoryFs;
use crate::adapters::git::{CommitSummary, GitGateway, ProductionGitAdapter};
use crate::domain::evidence::{Evidence, EvidenceSourceType};
use crate::domain::ledger::{ChangeLedgerEvent, ChangeStatus};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoStatusInput {
    #[serde(default)]
    pub include_recent_commits: Option<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoStatusOutput {
    pub branch: String,
    pub head_commit: String,
    pub working_tree_clean: bool,
    pub changed_files: Vec<String>,
    pub recent_commits: Vec<CommitSummary>,
    pub last_verified_commit: Option<String>,
    pub evidence: Vec<Evidence>,
}

fn clamp_include_recent_commits(input: Option<u8>) -> u8 {
    match input {
        Some(0) => 0,
        Some(n) if n > 50 => 50,
        Some(n) => n,
        None => 10,
    }
}

fn repo_rel_path(repo_root: &std::path::Path, absolute: &std::path::Path) -> Option<String> {
    absolute
        .strip_prefix(repo_root)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

async fn load_last_verified_ledger_event(
    fs: &SafeRepositoryFs,
) -> Result<Option<ChangeLedgerEvent>> {
    let ledger_path = "docs/work/CHANGELOG.ndjson";
    if !fs.exists(ledger_path)? {
        return Ok(None);
    }
    let contents = fs.read(ledger_path)?;
    let mut last_verified: Option<ChangeLedgerEvent> = None;
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<ChangeLedgerEvent>(line) {
            if matches!(event.status, ChangeStatus::Verified) {
                last_verified = Some(event);
            }
        }
    }
    Ok(last_verified)
}

pub async fn repo_status_impl(
    repo_root: PathBuf,
    include_recent_commits: Option<u8>,
) -> Result<RepoStatusOutput> {
    anyhow::ensure!(
        repo_root.exists(),
        "repo root does not exist: {}",
        repo_root.display()
    );
    let git = ProductionGitAdapter::new(repo_root.clone());

    let branch = git.branch().await?;
    let head_commit = git.head_commit().await?;
    let status = git.status_porcelain().await?;
    let max_count = clamp_include_recent_commits(include_recent_commits);
    let recent_commits = git.log(max_count).await?;

    let working_tree_clean = status.is_empty();
    let mut changed_files = Vec::new();
    for (_, raw_path) in status {
        if let Some(rel) = repo_rel_path(&repo_root, std::path::Path::new(&raw_path)) {
            changed_files.push(rel);
        } else {
            changed_files.push(raw_path);
        }
    }
    changed_files.sort();
    changed_files.dedup();

    let fs = SafeRepositoryFs::new(repo_root.clone());
    let verified_event = load_last_verified_ledger_event(&fs).await?;
    let last_verified_commit = verified_event.as_ref().and_then(|e| e.commit.clone());

    let evidence = vec![
        Evidence {
            source_type: EvidenceSourceType::Git,
            path: "branch".to_string(),
            revision: Some(head_commit.clone()),
            extracted_at: chrono::Utc::now().to_rfc3339(),
        },
        Evidence {
            source_type: EvidenceSourceType::Git,
            path: "status".to_string(),
            revision: Some(head_commit.clone()),
            extracted_at: chrono::Utc::now().to_rfc3339(),
        },
        Evidence {
            source_type: EvidenceSourceType::Ledger,
            path: "docs/work/CHANGELOG.ndjson".to_string(),
            revision: last_verified_commit.clone(),
            extracted_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    Ok(RepoStatusOutput {
        branch,
        head_commit,
        working_tree_clean,
        changed_files,
        recent_commits,
        last_verified_commit,
        evidence,
    })
}

pub struct RepoStatusTool {
    repo_root: PathBuf,
}

impl RepoStatusTool {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[async_trait]
impl ToolHandler for RepoStatusTool {
    async fn handle(
        &self,
        args: serde_json::Value,
        _extra: RequestHandlerExtra,
    ) -> McpResult<serde_json::Value> {
        let input: RepoStatusInput = serde_json::from_value(args)
            .map_err(|e| McpError::validation(format!("Invalid arguments: {}", e)))?;

        let output = repo_status_impl(self.repo_root.clone(), input.include_recent_commits)
            .await
            .map_err(|e| McpError::validation(e.to_string()))?;

        serde_json::to_value(output)
            .map_err(|e| McpError::validation(format!("Failed to serialize output: {}", e)))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo::new(
            "repo_status",
            Some("Return repository startup evidence including branch, head commit, working tree status, recent commits, and last verified ledger event.".to_string()),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "include_recent_commits": {
                        "type": "integer",
                        "description": "Number of recent commits to include (default 10, max 50)",
                        "minimum": 0,
                        "maximum": 50
                    }
                }
            }),
        ))
    }
}
