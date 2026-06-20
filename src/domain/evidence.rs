use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSourceType {
    Git,
    TaskContract,
    Ledger,
    GeneratedArtifact,
    Invariant,
    TestOutput,
    CargoMetadata,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub source_type: EvidenceSourceType,
    pub path: String,
    pub revision: Option<String>,
    pub extracted_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub command: String,
    pub passed: bool,
    pub executed_at: String,
}

impl ValidationResult {
    pub fn passed(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            passed: true,
            executed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricBucket {
    ToolInvocations,
    ValidationResults,
    TaskCompletion,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskMetrics {
    pub task_id: String,
    pub tool_invocations: u64,
    pub validations_passed: u64,
    pub validations_failed: u64,
    pub completion_status: Option<String>,
}
