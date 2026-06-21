use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error as McpError, RequestHandlerExtra, Result as McpResult, ToolHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::adapters::filesystem::SafeRepositoryFs;
use crate::adapters::git::{GitGateway, ProductionGitAdapter};
use crate::domain::evidence::{Evidence, EvidenceSourceType};
use crate::domain::ledger::ChangeLedgerEvent;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextBundleInput {
    pub task_id: String,
    #[schemars(range(min = 0, max = 50))]
    #[serde(default = "default_ledger_limit")]
    pub recent_ledger_limit: u8,
    #[serde(default)]
    pub include_diff: bool,
    #[serde(default)]
    pub include_dependency_neighborhood: bool,
}

fn default_ledger_limit() -> u8 {
    10
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelatedFile {
    pub path: String,
    pub reason: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Invariant {
    pub description: String,
    pub rule: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryState {
    pub branch: String,
    pub head_commit: String,
    pub working_tree_clean: bool,
    pub changed_files: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextBundleOutput {
    pub task: crate::domain::task::TaskContract,
    pub repository_state: RepositoryState,
    pub current_snapshot: String,
    pub recent_events: Vec<ChangeLedgerEvent>,
    pub changed_files: Vec<String>,
    pub related_files: Vec<RelatedFile>,
    pub invariants: Vec<Invariant>,
    pub required_validations: Vec<String>,
    pub evidence: Vec<Evidence>,
}

fn load_task_contract(fs: &SafeRepositoryFs, task_id: &str) -> Result<crate::domain::task::TaskContract> {
    let json_path = format!("docs/work/tasks/{task_id}.json");
    if !fs.exists(&json_path)? {
        anyhow::bail!("task contract not found: {json_path}");
    }
    let contents = fs.read(&json_path)?;
    let contract = crate::domain::task::TaskContract::parse_from_json(&contents)?;
    contract.validate()?;
    Ok(contract)
}

async fn load_recent_ledger_events(
    fs: &SafeRepositoryFs,
    limit: u8,
) -> Result<Vec<ChangeLedgerEvent>> {
    let ledger_path = "docs/work/CHANGELOG.ndjson";
    if !fs.exists(ledger_path)? {
        return Ok(Vec::new());
    }
    let contents = fs.read(ledger_path)?;
    let mut events = Vec::new();
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<ChangeLedgerEvent>(line) {
            events.push(event);
        }
    }
    events.reverse();
    events.truncate(limit as usize);
    Ok(events)
}

pub async fn get_context_bundle_impl(
    repo_root: PathBuf,
    input: ContextBundleInput,
) -> Result<ContextBundleOutput> {
    anyhow::ensure!(repo_root.exists(), "repo root does not exist: {}", repo_root.display());
    let git = ProductionGitAdapter::new(repo_root.clone());

    let branch = git.branch().await?;
    let head_commit = git.head_commit().await?;
    let status = git.status_porcelain().await?;
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
    let task = load_task_contract(&fs, &input.task_id)?;
    let recent_events = load_recent_ledger_events(&fs, input.recent_ledger_limit).await?;
    let related_files = Vec::new();
    let current_snapshot = head_commit.clone();
    let invariants = task.invariants.clone().into_iter().map(|desc| Invariant {
        description: desc.clone(),
        rule: "from task contract".to_string(),
    }).collect();
    let required_validations = vec![
        "clippy".to_string(),
        "cargo test".to_string(),
    ];

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
            source_type: EvidenceSourceType::TaskContract,
            path: format!("docs/work/tasks/{}.json", input.task_id),
            revision: None,
            extracted_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    Ok(ContextBundleOutput {
        task,
        repository_state: RepositoryState {
            branch,
            head_commit: head_commit.clone(),
            working_tree_clean,
            changed_files: changed_files.clone(),
        },
        current_snapshot,
        recent_events,
        changed_files,
        related_files,
        invariants,
        required_validations,
        evidence,
    })
}

fn repo_rel_path(repo_root: &std::path::Path, absolute: &std::path::Path) -> Option<String> {
    absolute
        .strip_prefix(repo_root)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

pub struct ContextBundleTool {
    repo_root: PathBuf,
}

impl ContextBundleTool {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bundle_requires_existing_task_contract() {
        let dir = tempfile::tempdir().unwrap();
        let fs = SafeRepositoryFs::new(dir.path().to_path_buf());
        let result = load_task_contract(&fs, "MISSING");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn ledger_returns_empty_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let fs = SafeRepositoryFs::new(dir.path().to_path_buf());
        let events = load_recent_ledger_events(&fs, 10).await.unwrap();
        assert!(events.is_empty());
    }
}

#[async_trait]
impl ToolHandler for ContextBundleTool {
    async fn handle(
        &self,
        args: serde_json::Value,
        _extra: RequestHandlerExtra,
    ) -> McpResult<serde_json::Value> {
        let input: ContextBundleInput = serde_json::from_value(args)
            .map_err(|e| McpError::validation(format!("Invalid arguments: {}", e)))?;

        let output = get_context_bundle_impl(self.repo_root.clone(), input)
            .await
            .map_err(|e| McpError::validation(e.to_string()))?;

        serde_json::to_value(output)
            .map_err(|e| McpError::validation(format!("Failed to serialize output: {}", e)))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo::new(
            "get_context_bundle",
            Some("Load task-scoped repository knowledge: task contract, recent ledger events, repo state, and evidence for the requested task.".to_string()),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "Target task ID (e.g., TASK-001)"
                    },
                    "recent_ledger_limit": {
                        "type": "integer",
                        "description": "Max recent ledger events to load (default 10, max 50)",
                        "minimum": 0,
                        "maximum": 50
                    },
                    "include_diff": {
                        "type": "boolean",
                        "description": "Include current git diff",
                        "default": false
                    },
                    "include_dependency_neighborhood": {
                        "type": "boolean",
                        "description": "Include dependency neighborhood",
                        "default": false
                    }
                },
                "required": ["task_id"]
            }),
        ))
    }
}
