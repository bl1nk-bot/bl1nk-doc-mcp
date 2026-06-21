use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImpactSeverity {
    Minor,
    Major,
    Breaking,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequiredActionKind {
    RegenerateOpenApi,
    RegenerateSchemaDocs,
    RegenerateModuleMap,
    UpdateTaskContract,
    AppendLedgerEvent,
    ReviewSecurityInvariant,
    ReviewAdr,
    RunMigrationTest,
    RunAffectedTests,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequiredAction {
    pub kind: RequiredActionKind,
    pub target: String,
    pub reason: String,
    pub required: bool,
    pub status: ActionStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangeClassification {
    pub classification_path: String,
    pub rule_set: Vec<String>,
    pub reasons: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeImpactOutput {
    pub change_id: String,
    pub changed_files: Vec<String>,
    pub classifications: Vec<ChangeClassification>,
    pub required_actions: Vec<RequiredAction>,
    pub blocking_actions: Vec<RequiredAction>,
    pub evidence: Vec<crate::domain::evidence::Evidence>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeImpactInput {
    pub base_ref: String,
    pub head_ref: String,
    #[serde(default)]
    pub task_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImpactAnalysis {
    pub task_id: String,
    pub base_ref: String,
    pub head_ref: String,
    pub changed_files: Vec<PathBuf>,
    pub required_actions: Vec<RequiredAction>,
    pub blocking_actions: Vec<RequiredAction>,
}
