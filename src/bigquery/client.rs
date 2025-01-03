use super::{queries::*, BigQueryConfig, BigQueryError};
use crate::models::{BigQueryAction, ProposalArgs, VoteArgs};
use serde_json::Value;
use std::path::Path;
use tracing::{error, info};

pub struct BigQueryClient {
    project_id: String,
    // TODO: Add GCP BigQuery client when we have credentials
}

impl BigQueryClient {
    pub async fn new(config: BigQueryConfig) -> Result<Self, BigQueryError> {
        // Validate credentials file exists
        if !Path::new(&config.credentials_path).exists() {
            return Err(BigQueryError::Auth(format!(
                "Credentials file not found: {}",
                config.credentials_path
            )));
        }

        Ok(Self {
            project_id: config.project_id,
        })
    }

    pub async fn fetch_dao_accounts(&self) -> Result<Vec<String>, BigQueryError> {
        info!("Fetching DAO accounts");

        // TODO: Execute QUERY_DAO_ACCOUNTS when we have credentials
        // For now, return empty vec to allow development of other components
        Ok(vec![])
    }

    pub async fn fetch_dao_proposals(
        &self,
        dao_id: &str,
        start_block: Option<i64>,
    ) -> Result<Vec<BigQueryAction>, BigQueryError> {
        info!("Fetching proposals for DAO: {}", dao_id);

        let query = query_dao_proposals(dao_id, start_block);
        // TODO: Execute query when we have credentials
        Ok(vec![])
    }

    pub async fn fetch_proposal_votes(
        &self,
        dao_id: &str,
        start_block: Option<i64>,
    ) -> Result<Vec<BigQueryAction>, BigQueryError> {
        info!("Fetching votes for DAO: {}", dao_id);

        let query = query_dao_votes(dao_id, start_block);
        // TODO: Execute query when we have credentials
        Ok(vec![])
    }

    // Helper function to parse proposal kind
    pub fn parse_proposal_kind(&self, proposal: &Value) -> Result<String, BigQueryError> {
        let kind = proposal
            .get("kind")
            .ok_or_else(|| BigQueryError::Parse("Missing 'kind' field".to_string()))?;

        match kind {
            Value::Object(map) => {
                if map.contains_key("AddMemberToRole") {
                    Ok("AddMember".to_string())
                } else if map.contains_key("RemoveMemberFromRole") {
                    Ok("RemoveMember".to_string())
                } else if map.contains_key("FunctionCall") {
                    Ok("FunctionCall".to_string())
                } else if map.contains_key("Transfer") {
                    Ok("Transfer".to_string())
                } else if map.contains_key("Vote") {
                    Ok("Vote".to_string())
                } else {
                    Err(BigQueryError::Parse("Unknown proposal kind".to_string()))
                }
            }
            _ => Err(BigQueryError::Parse(
                "Invalid proposal kind format".to_string(),
            )),
        }
    }

    // Helper function to extract member details from proposal
    pub fn extract_member_details(
        &self,
        proposal: &Value,
    ) -> Result<(String, String), BigQueryError> {
        let kind = proposal
            .get("kind")
            .and_then(|k| k.as_object())
            .ok_or_else(|| BigQueryError::Parse("Invalid proposal kind".to_string()))?;

        if let Some(add_member) = kind.get("AddMemberToRole") {
            let member_id = add_member
                .get("member_id")
                .and_then(|m| m.as_str())
                .ok_or_else(|| BigQueryError::Parse("Missing member_id".to_string()))?;
            let role = add_member
                .get("role")
                .and_then(|r| r.as_str())
                .ok_or_else(|| BigQueryError::Parse("Missing role".to_string()))?;
            Ok((member_id.to_string(), role.to_string()))
        } else if let Some(remove_member) = kind.get("RemoveMemberFromRole") {
            let member_id = remove_member
                .get("member_id")
                .and_then(|m| m.as_str())
                .ok_or_else(|| BigQueryError::Parse("Missing member_id".to_string()))?;
            let role = remove_member
                .get("role")
                .and_then(|r| r.as_str())
                .ok_or_else(|| BigQueryError::Parse("Missing role".to_string()))?;
            Ok((member_id.to_string(), role.to_string()))
        } else {
            Err(BigQueryError::Parse("Not a member proposal".to_string()))
        }
    }

    // Helper function to extract transfer details
    pub fn extract_transfer_details(
        &self,
        proposal: &Value,
    ) -> Result<(String, String, String, Option<String>), BigQueryError> {
        let transfer = proposal
            .get("kind")
            .and_then(|k| k.get("Transfer"))
            .ok_or_else(|| BigQueryError::Parse("Not a transfer proposal".to_string()))?;

        let token_id = transfer
            .get("token_id")
            .and_then(|t| t.as_str())
            .ok_or_else(|| BigQueryError::Parse("Missing token_id".to_string()))?;
        let receiver_id = transfer
            .get("receiver_id")
            .and_then(|r| r.as_str())
            .ok_or_else(|| BigQueryError::Parse("Missing receiver_id".to_string()))?;
        let amount = transfer
            .get("amount")
            .and_then(|a| a.as_str())
            .ok_or_else(|| BigQueryError::Parse("Missing amount".to_string()))?;
        let msg = transfer
            .get("msg")
            .and_then(|m| m.as_str())
            .map(String::from);

        Ok((
            token_id.to_string(),
            receiver_id.to_string(),
            amount.to_string(),
            msg,
        ))
    }

    // Helper function to extract function call details
    pub fn extract_function_call_details(
        &self,
        proposal: &Value,
    ) -> Result<(String, Value), BigQueryError> {
        let function_call = proposal
            .get("kind")
            .and_then(|k| k.get("FunctionCall"))
            .ok_or_else(|| BigQueryError::Parse("Not a function call proposal".to_string()))?;

        let receiver_id = function_call
            .get("receiver_id")
            .and_then(|r| r.as_str())
            .ok_or_else(|| BigQueryError::Parse("Missing receiver_id".to_string()))?;
        let actions = function_call
            .get("actions")
            .ok_or_else(|| BigQueryError::Parse("Missing actions".to_string()))?;

        Ok((receiver_id.to_string(), actions.clone()))
    }
}
