use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error as McpError, RequestHandlerExtra, Result as McpResult, ToolHandler};

use crate::adapters::git::{GitGateway, ProductionGitAdapter};
use crate::domain::evidence::{Evidence, EvidenceSourceType};
use crate::domain::impact::{
    AnalyzeImpactInput, AnalyzeImpactOutput, ChangeClassification, RequiredAction,
    RequiredActionKind,
};

#[allow(dead_code)]
pub async fn analyze_change_impact_impl(
    repo_root: PathBuf,
    input: AnalyzeImpactInput,
) -> Result<AnalyzeImpactOutput> {
    anyhow::ensure!(
        repo_root.exists(),
        "repo root does not exist: {}",
        repo_root.display()
    );
    let git = ProductionGitAdapter::new(repo_root.clone());

    let base = input.base_ref.trim();
    let head = input.head_ref.trim();
    anyhow::ensure!(!base.is_empty(), "base_ref must not be empty");
    anyhow::ensure!(!head.is_empty(), "head_ref must not be empty");

    let changed_files = git
        .diff_names(base, head)
        .await
        .map_err(|e| anyhow::anyhow!("failed to compute diff: {e}"))?;

    let (mut classifications, actions) = classify_changes(&changed_files);
    if classifications.is_empty() {
        classifications.push(ChangeClassification {
            classification_path: "other".to_string(),
            rule_set: vec!["default_no_match".to_string()],
            reasons: vec!["no known classification matched".to_string()],
        });
    }

    let change_id = format!("{base}..{head}");
    let head_commit = git.head_commit().await.unwrap_or_default();
    let evidence = vec![
        Evidence {
            source_type: EvidenceSourceType::Git,
            path: "diff".to_string(),
            revision: Some(head_commit),
            extracted_at: chrono::Utc::now().to_rfc3339(),
        },
        Evidence {
            source_type: EvidenceSourceType::Git,
            path: format!("diff {}..{}", base, head),
            revision: None,
            extracted_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    Ok(AnalyzeImpactOutput {
        change_id,
        changed_files: changed_files.clone(),
        classifications,
        required_actions: actions.clone(),
        blocking_actions: Vec::new(),
        evidence,
    })
}

fn classify_changes(files: &[String]) -> (Vec<ChangeClassification>, Vec<RequiredAction>) {
    let mut classifications = Vec::new();
    let mut seen_actions = std::collections::HashSet::new();
    let mut actions = Vec::new();

    for file in files {
        let path = file.as_str();
        if path.starts_with("migrations/") && path.ends_with(".sql") {
            classifications.push(ChangeClassification {
                classification_path: "migration_path".to_string(),
                rule_set: vec!["migrations/**".to_string()],
                reasons: vec![format!("migration file changed: {file}")],
            });
            for kind in [
                RequiredActionKind::RegenerateSchemaDocs,
                RequiredActionKind::RunMigrationTest,
            ] {
                if seen_actions.insert(format!("{:?}", kind)) {
                    actions.push(RequiredAction {
                        kind,
                        target: file.clone(),
                        reason: format!("migration changed: {file}"),
                        required: true,
                        status: crate::domain::impact::ActionStatus::Pending,
                    });
                }
            }
        } else if path.starts_with("src/routes/") {
            classifications.push(ChangeClassification {
                classification_path: "route_path".to_string(),
                rule_set: vec!["src/routes/**".to_string()],
                reasons: vec![format!("route file changed: {file}")],
            });
            for kind in [
                RequiredActionKind::RegenerateOpenApi,
                RequiredActionKind::RunAffectedTests,
            ] {
                if seen_actions.insert(format!("{:?}", kind)) {
                    actions.push(RequiredAction {
                        kind,
                        target: file.clone(),
                        reason: format!("route changed: {file}"),
                        required: true,
                        status: crate::domain::impact::ActionStatus::Pending,
                    });
                }
            }
        } else if path.starts_with("src/auth/") {
            classifications.push(ChangeClassification {
                classification_path: "auth_path".to_string(),
                rule_set: vec!["src/auth/**".to_string()],
                reasons: vec![format!("auth file changed: {file}")],
            });
            let kind = RequiredActionKind::ReviewSecurityInvariant;
            if seen_actions.insert(format!("{:?}", kind)) {
                actions.push(RequiredAction {
                    kind,
                    target: file.clone(),
                    reason: format!("auth changed: {file}"),
                    required: true,
                    status: crate::domain::impact::ActionStatus::Pending,
                });
            }
            if seen_actions.insert("RunAffectedTests".to_string()) {
                actions.push(RequiredAction {
                    kind: RequiredActionKind::RunAffectedTests,
                    target: file.clone(),
                    reason: format!("auth changed: {file}"),
                    required: true,
                    status: crate::domain::impact::ActionStatus::Pending,
                });
            }
        } else if path == "Cargo.toml" {
            classifications.push(ChangeClassification {
                classification_path: "crate_path".to_string(),
                rule_set: vec!["Cargo.toml".to_string()],
                reasons: vec!["Cargo.toml changed".to_string()],
            });
            let kind = RequiredActionKind::RegenerateModuleMap;
            if seen_actions.insert(format!("{:?}", kind)) {
                actions.push(RequiredAction {
                    kind,
                    target: "Cargo.toml".to_string(),
                    reason: "crate manifest changed".to_string(),
                    required: true,
                    status: crate::domain::impact::ActionStatus::Pending,
                });
            }
            if seen_actions.insert("RunAffectedTests".to_string()) {
                actions.push(RequiredAction {
                    kind: RequiredActionKind::RunAffectedTests,
                    target: "Cargo.toml".to_string(),
                    reason: "crate manifest changed".to_string(),
                    required: true,
                    status: crate::domain::impact::ActionStatus::Pending,
                });
            }
        }
    }

    (classifications, actions)
}

pub struct AnalyzeImpactTool {
    repo_root: PathBuf,
}

impl AnalyzeImpactTool {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_changes_detects_migration() {
        let (classifications, actions) = classify_changes(&["migrations/001.sql".to_string()]);
        assert!(classifications.iter().any(|c| c.classification_path == "migration_path"));
        assert!(actions.iter().any(|a| {
            matches!(
                a.kind,
                RequiredActionKind::RegenerateSchemaDocs | RequiredActionKind::RunMigrationTest
            )
        }));
    }

    #[test]
    fn classify_changes_detects_cargo_toml() {
        let (classifications, actions) = classify_changes(&["Cargo.toml".to_string()]);
        assert!(classifications.iter().any(|c| c.classification_path == "crate_path"));
        assert!(actions.iter().any(|a| matches!(
            a.kind,
            RequiredActionKind::RegenerateModuleMap
        ) || matches!(
            a.kind,
            RequiredActionKind::RunAffectedTests
        )));
    }

    #[test]
    fn classify_changes_no_match() {
        let (classifications, actions) = classify_changes(&["README.md".to_string()]);
        assert!(classifications.is_empty());
        assert!(actions.is_empty());
    }
}

#[async_trait]
impl ToolHandler for AnalyzeImpactTool {
    async fn handle(
        &self,
        args: serde_json::Value,
        _extra: RequestHandlerExtra,
    ) -> McpResult<serde_json::Value> {
        let input: AnalyzeImpactInput = serde_json::from_value(args)
            .map_err(|e| McpError::validation(format!("Invalid arguments: {}", e)))?;

        let output = analyze_change_impact_impl(self.repo_root.clone(), input)
            .await
            .map_err(|e| McpError::validation(e.to_string()))?;

        serde_json::to_value(output)
            .map_err(|e| McpError::validation(format!("Failed to serialize output: {}", e)))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo::new(
            "analyze_change_impact",
            Some("Analyze the impact of changes between two refs and produce classification, required actions, and evidence.".to_string()),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "base_ref": {
                        "type": "string",
                        "description": "Base ref (commit, tag, or branch)"
                    },
                    "head_ref": {
                        "type": "string",
                        "description": "Head ref (commit, tag, or branch)"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Optional associated task id",
                        "default": null
                    }
                },
                "required": ["base_ref", "head_ref"]
            }),
        ))
    }
}
