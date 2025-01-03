use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dao {
    pub id: i32,
    pub account_id: String, // e.g. "example.sputnik-dao.near"
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProposalType {
    AddMember,    // AddMemberToRole
    RemoveMember, // RemoveMemberFromRole
    FunctionCall,
    Transfer,
    Vote,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Proposal {
    pub id: i32,
    pub dao_id: i32,
    pub proposal_id: i64,      // Original proposal ID from the contract
    pub proposer: String,      // Account that created the proposal
    pub proposal_type: String, // Stored as string in DB, converted to ProposalType enum
    pub description: String,
    pub status: String, // "InProgress", "Approved", "Rejected", "Removed"

    // Specific fields based on proposal type
    pub target_member: Option<String>, // For AddMember/RemoveMember
    pub target_role: Option<String>,   // For AddMember/RemoveMember
    pub receiver_id: Option<String>,   // For FunctionCall/Transfer
    pub actions: Option<serde_json::Value>, // For FunctionCall details
    pub token_id: Option<String>,      // For Transfer
    pub amount: Option<String>,        // For Transfer (as string to handle U128)

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Member {
    pub id: i32,
    pub dao_id: i32,
    pub account_id: String,
    pub roles: Vec<String>, // Array of roles the member has
    pub added_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: i32,
    pub proposal_id: i32,
    pub voter: String,
    pub vote: String, // "Approve", "Reject", "Remove"
    pub voted_at: chrono::DateTime<chrono::Utc>,
}

// BigQuery specific structs for parsing query results

#[derive(Debug, Deserialize)]
pub struct BigQueryAction {
    pub block_timestamp: i64,
    pub signer_account_id: String,
    pub receiver_account_id: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ProposalArgs {
    pub proposal: serde_json::Value,
    pub proposal_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct VoteArgs {
    pub proposal_id: i64,
    pub vote: String,
}
