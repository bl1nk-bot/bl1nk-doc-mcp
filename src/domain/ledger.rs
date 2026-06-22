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
    pub fn new(
        task_id: impl Into<String>,
        intent: impl Into<String>,
        status: ChangeStatus,
    ) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event_has_generated_fields() {
        let event = ChangeLedgerEvent::new("TASK-1", "fix bug", ChangeStatus::Draft);
        assert!(!event.id.is_empty());
        assert!(!event.timestamp.is_empty());
        assert!(event.commit.is_none());
        assert_eq!(event.task_id, "TASK-1");
        assert_eq!(event.intent, "fix bug");
        assert!(event.scope.is_empty());
        assert!(event.changed_contracts.is_empty());
        assert!(event.invariants_added.is_empty());
        assert!(event.validations.is_empty());
    }

    #[test]
    fn is_valid_rejects_empty_scope() {
        let event = ChangeLedgerEvent::new("TASK-1", "fix", ChangeStatus::Draft);
        assert!(!event.is_valid(), "empty scope should be invalid");
    }

    #[test]
    fn is_valid_accepts_non_empty_scope_draft() {
        let mut event = ChangeLedgerEvent::new("TASK-1", "fix", ChangeStatus::Draft);
        event.scope = vec!["src/main.rs".to_string()];
        assert!(event.is_valid(), "draft with scope should be valid");
    }

    #[test]
    fn is_valid_rejects_verified_without_passing_validations() {
        let mut event = ChangeLedgerEvent::new("TASK-1", "fix", ChangeStatus::Verified);
        event.scope = vec!["src/main.rs".to_string()];
        event.validations = vec![crate::domain::evidence::ValidationResult {
            command: "clippy".to_string(),
            passed: false,
            executed_at: chrono::Utc::now().to_rfc3339(),
        }];
        assert!(
            !event.is_valid(),
            "verified without passing validation should be invalid"
        );
    }

    #[test]
    fn is_valid_accepts_verified_with_passing_validations() {
        let mut event = ChangeLedgerEvent::new("TASK-1", "fix", ChangeStatus::Verified);
        event.scope = vec!["src/main.rs".to_string()];
        event.validations = vec![crate::domain::evidence::ValidationResult {
            command: "clippy".to_string(),
            passed: true,
            executed_at: chrono::Utc::now().to_rfc3339(),
        }];
        assert!(event.is_valid(), "verified with passing validation should be valid");
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AppendLedgerInput {
    pub task_id: String,
    pub intent: String,
    pub scope: Vec<String>,
    pub changed_contracts: Vec<String>,
    #[serde(default)]
    pub invariants_added: Vec<String>,
    pub validations: Vec<crate::domain::evidence::ValidationResult>,
    #[serde(rename = "status")]
    pub status: ChangeStatus,
    pub commit: Option<String>,
}

impl From<AppendLedgerInput> for ChangeLedgerEvent {
    fn from(value: AppendLedgerInput) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            commit: value.commit,
            task_id: value.task_id,
            scope: value.scope,
            intent: value.intent,
            changed_contracts: value.changed_contracts,
            invariants_added: value.invariants_added,
            validations: value.validations,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AppendLedgerOutput {
    pub id: String,
    pub timestamp: String,
    pub status: ChangeStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LedgerAppendError {
    pub reason: String,
}
