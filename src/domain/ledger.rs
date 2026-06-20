use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Draft,
    Verified,
    Blocked,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangeLedgerEvent {
    pub id: String,
    pub timestamp: String,
    pub commit: Option<String>,
    pub task_id: String,
    pub scope: Vec<String>,
    pub intent: String,
    #[serde(default)]
    pub changed_contracts: Vec<String>,
    #[serde(default)]
    pub invariants_added: Vec<String>,
    #[serde(default)]
    pub validations: Vec<crate::domain::evidence::ValidationResult>,
    pub status: ChangeStatus,
}

impl ChangeLedgerEvent {
    pub fn new(task_id: impl Into<String>, intent: impl Into<String>, status: ChangeStatus) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            commit: None,
            task_id: task_id.into(),
            scope: Vec::new(),
            intent: intent.into(),
            changed_contracts: Vec::new(),
            invariants_added: Vec::new(),
            validations: Vec::new(),
            status,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.scope.is_empty() {
            return false;
        }
        if matches!(self.status, ChangeStatus::Verified)
            && !self.validations.iter().any(|v| v.passed)
        {
            return false;
        }
        true
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LedgerAppendError {
    pub reason: String,
}
