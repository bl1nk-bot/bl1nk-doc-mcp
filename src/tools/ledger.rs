use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error as McpError, RequestHandlerExtra, Result as McpResult, ToolHandler};

use crate::domain::ledger::{AppendLedgerInput, AppendLedgerOutput, ChangeLedgerEvent};

pub async fn append_change_ledger_impl(
    repo_root: PathBuf,
    input: AppendLedgerInput,
) -> Result<AppendLedgerOutput> {
    anyhow::ensure!(!input.task_id.trim().is_empty(), "task_id must not be empty");
    anyhow::ensure!(!input.intent.trim().is_empty(), "intent must not be empty");
    anyhow::ensure!(repo_root.exists(), "repo root does not exist: {}", repo_root.display());
    let parent = repo_root.join("docs/work");
    std::fs::create_dir_all(&parent)
        .with_context(|| format!("failed to create docs/work: {}", repo_root.display()))?;
    let event: ChangeLedgerEvent = input.into();
    let line = serde_json::to_string(&event)
        .map_err(|e| anyhow::anyhow!("failed to serialize ledger event: {e}"))?;
    let ledger_path = parent.join("CHANGELOG.ndjson");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ledger_path)
        .with_context(|| format!("failed to open ledger: {}", ledger_path.display()))?
        .write_all(line.as_bytes())
        .map(|_| ())
        .with_context(|| format!("failed to write ledger: {}", ledger_path.display()))?;

    Ok(AppendLedgerOutput {
        id: event.id,
        timestamp: event.timestamp,
        status: event.status,
    })
}

pub struct AppendLedgerTool {
    repo_root: PathBuf,
}

impl AppendLedgerTool {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn event_requires_task_id() {
        let dir = tempdir().unwrap();
        let input = AppendLedgerInput {
            task_id: "".to_string(),
            intent: "fix bug".to_string(),
            scope: vec![],
            changed_contracts: vec![],
            invariants_added: vec![],
            validations: vec![],
            status: crate::domain::ledger::ChangeStatus::Draft,
            commit: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(append_change_ledger_impl(dir.path().to_path_buf(), input))
            .unwrap_err();
        assert!(err.to_string().contains("task_id"));
    }

    #[test]
    fn event_requires_intent() {
        let dir = tempdir().unwrap();
        let input = AppendLedgerInput {
            task_id: "TASK-1".to_string(),
            intent: "".to_string(),
            scope: vec!["src/main.rs".to_string()],
            changed_contracts: vec![],
            invariants_added: vec![],
            validations: vec![],
            status: crate::domain::ledger::ChangeStatus::Draft,
            commit: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(append_change_ledger_impl(dir.path().to_path_buf(), input))
            .unwrap_err();
        assert!(err.to_string().contains("intent"));
    }

    #[test]
    fn append_writes_ndjson_line() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("docs/work")).unwrap();
        let input = AppendLedgerInput {
            task_id: "TASK-1".to_string(),
            intent: "fix bug".to_string(),
            scope: vec!["src/main.rs".to_string()],
            changed_contracts: vec!["src/main.rs".to_string()],
            invariants_added: vec![],
            validations: vec![],
            status: crate::domain::ledger::ChangeStatus::Draft,
            commit: Some("abc123".to_string()),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let output = rt
            .block_on(append_change_ledger_impl(dir.path().to_path_buf(), input))
            .unwrap();
        assert!(!output.id.is_empty());
        assert!(std::path::Path::new(&dir.path().join("docs/work/CHANGELOG.ndjson")).exists());
    }
}

#[async_trait]
impl ToolHandler for AppendLedgerTool {
    async fn handle(
        &self,
        args: serde_json::Value,
        _extra: RequestHandlerExtra,
    ) -> McpResult<serde_json::Value> {
        let input: AppendLedgerInput = serde_json::from_value(args)
            .map_err(|e| McpError::validation(format!("Invalid arguments: {}", e)))?;

        let output = append_change_ledger_impl(self.repo_root.clone(), input)
            .await
            .map_err(|e| McpError::validation(e.to_string()))?;

        serde_json::to_value(output)
            .map_err(|e| McpError::validation(format!("Failed to serialize output: {}", e)))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo::new(
            "append_change_ledger",
            Some("Append a new change ledger event to CHANGELOG.ndjson.".to_string()),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Associated task id"},
                    "intent": {"type": "string", "description": "Short intent description"},
                    "status": {
                        "type": "string",
                        "enum": ["draft", "verified", "blocked"],
                        "description": "Ledger event status"
                    },
                    "scope": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Files or contracts affected"
                    },
                    "changed_contracts": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Contracts changed in this event"
                    },
                    "invariants_added": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "New invariants introduced"
                    },
                    "validations": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "command": {"type": "string"},
                                "passed": {"type": "boolean"},
                                "executed_at": {"type": "string"}
                            },
                            "required": ["command", "passed", "executed_at"]
                        },
                        "description": "Validation results attached to this event"
                    },
                    "commit": {
                        "type": "string",
                        "description": "Optional commit sha associated with the change"
                    }
                },
                "required": ["task_id", "intent", "status", "scope"]
            }),
        ))
    }
}
