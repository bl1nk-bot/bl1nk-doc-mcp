use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Pending,
    Running,
    Passed,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValidationFailure {
    PathTraversal,
    FileNotFound,
    InvalidContract,
    InvalidLedgerEvent,
    InvalidRepositoryRoot,
    Unexpected,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Validation {
    pub id: String,
    pub command: String,
    pub status: ValidationStatus,
    pub output: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckEvaluation {
    pub check_id: String,
    pub passed: bool,
    pub failure: Option<ValidationFailure>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequiredToolCall {
    pub tool: String,
    pub required: bool,
    pub called: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompletionScore {
    pub required_tool_calls_met: bool,
    pub impact_actions_resolved: bool,
    pub validations_passed: bool,
    pub overall_passed: bool,
}
