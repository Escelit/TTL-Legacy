use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: String,
    pub owner: String,
    pub beneficiary: String,
    pub balance: i128,
    pub check_in_interval: u64,
    pub last_check_in: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub status: VaultStatus,
    pub ttl_remaining: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VaultStatus {
    Active,
    Expired,
    Released,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEvent {
    pub vault_id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    CheckIn,
    TtlUpdate,
    StatusChange,
    Deposit,
    Withdrawal,
    Release,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub owner: Option<String>,
    pub beneficiary: Option<String>,
    pub status: Option<VaultStatus>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub vaults: Vec<Vault>,
    pub total: u32,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub vaults: Vec<Vault>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub vault: Vault,
    pub history: Vec<VaultEvent>,
    pub audit_log: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub actor: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub vault_id: String,
    pub owner: String,
    pub beneficiary: String,
    pub report_generated_at: DateTime<Utc>,
    pub fund_movements: Vec<FundMovement>,
    pub beneficiary_changes: Vec<BeneficiaryChange>,
    pub ttl_history: Vec<TtlEvent>,
    pub total_deposits: i128,
    pub total_withdrawals: i128,
    pub current_balance: i128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FundMovement {
    pub timestamp: DateTime<Utc>,
    pub movement_type: String,
    pub amount: i128,
    pub balance_after: i128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeneficiaryChange {
    pub timestamp: DateTime<Utc>,
    pub old_beneficiary: String,
    pub new_beneficiary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TtlEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub ttl_remaining: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub check_in_interval: u64,
    pub recommended_for: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultTemplateList {
    pub templates: Vec<VaultTemplate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVaultFromTemplate {
    pub template_id: String,
    pub owner: String,
    pub beneficiary: String,
}
