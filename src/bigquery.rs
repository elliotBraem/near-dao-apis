use crate::models::{BigQueryAction, ProposalArgs, VoteArgs};
use serde_json::Value;

const SPUTNIK_SUFFIX: &str = ".sputnik-dao.near";

pub struct BigQueryClient {
    // TODO: Add BigQuery client configuration
}

impl BigQueryClient {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn fetch_dao_accounts(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Query to find all *.sputnik-dao.near accounts
        let query = format!(
            r#"
        SELECT DISTINCT account_id
        FROM near_mainnet.accounts
        WHERE account_id LIKE '%{}'
        AND action_kind = 'CREATE_ACCOUNT'
        ORDER BY block_timestamp ASC
        "#,
            SPUTNIK_SUFFIX
        );

        // TODO: Execute query using credentials
        Ok(vec![])
    }

    pub async fn fetch_dao_proposals(
        &self,
        dao_id: &str,
        start_block: Option<i64>,
    ) -> Result<Vec<BigQueryAction>, Box<dyn std::error::Error>> {
        // Query to find all add_proposal actions
        let block_filter = start_block
            .map(|block| format!("AND block_timestamp >= {}", block))
            .unwrap_or_default();

        let query = format!(
            r#"
        SELECT
            block_timestamp,
            signer_account_id,
            receiver_account_id,
            args
        FROM near_mainnet.transaction_actions
        WHERE receiver_account_id = '{}'
        AND action_kind = 'FUNCTION_CALL'
        AND args:method_name = 'add_proposal'
        {}
        ORDER BY block_timestamp ASC
        "#,
            dao_id, block_filter
        );

        // TODO: Execute query using credentials
        Ok(vec![])
    }

    pub async fn fetch_proposal_votes(
        &self,
        dao_id: &str,
        start_block: Option<i64>,
    ) -> Result<Vec<BigQueryAction>, Box<dyn std::error::Error>> {
        // Query to find all act_proposal actions (votes)
        let block_filter = start_block
            .map(|block| format!("AND block_timestamp >= {}", block))
            .unwrap_or_default();

        let query = format!(
            r#"
        SELECT
            block_timestamp,
            signer_account_id,
            receiver_account_id,
            args
        FROM near_mainnet.transaction_actions
        WHERE receiver_account_id = '{}'
        AND action_kind = 'FUNCTION_CALL'
        AND args:method_name = 'act_proposal'
        {}
        ORDER BY block_timestamp ASC
        "#,
            dao_id, block_filter
        );

        // TODO: Execute query using credentials
        Ok(vec![])
    }

    // Helper function to parse proposal kind
    pub fn parse_proposal_kind(proposal: &Value) -> Option<String> {
        let kind = proposal.get("kind")?;

        match kind {
            Value::Object(map) => {
                if map.contains_key("AddMemberToRole") {
                    Some("AddMember".to_string())
                } else if map.contains_key("RemoveMemberFromRole") {
                    Some("RemoveMember".to_string())
                } else if map.contains_key("FunctionCall") {
                    Some("FunctionCall".to_string())
                } else if map.contains_key("Transfer") {
                    Some("Transfer".to_string())
                } else if map.contains_key("Vote") {
                    Some("Vote".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // Helper function to extract member details from proposal
    pub fn extract_member_details(proposal: &Value) -> Option<(String, String)> {
        let kind = proposal.get("kind")?.as_object()?;

        if let Some(add_member) = kind.get("AddMemberToRole") {
            Some((
                add_member.get("member_id")?.as_str()?.to_string(),
                add_member.get("role")?.as_str()?.to_string(),
            ))
        } else if let Some(remove_member) = kind.get("RemoveMemberFromRole") {
            Some((
                remove_member.get("member_id")?.as_str()?.to_string(),
                remove_member.get("role")?.as_str()?.to_string(),
            ))
        } else {
            None
        }
    }

    // Helper function to extract transfer details
    pub fn extract_transfer_details(
        proposal: &Value,
    ) -> Option<(String, String, String, Option<String>)> {
        let transfer = proposal.get("kind")?.get("Transfer")?;

        Some((
            transfer.get("token_id")?.as_str()?.to_string(),
            transfer.get("receiver_id")?.as_str()?.to_string(),
            transfer.get("amount")?.as_str()?.to_string(),
            transfer
                .get("msg")
                .and_then(|m| m.as_str())
                .map(String::from),
        ))
    }

    // Helper function to extract function call details
    pub fn extract_function_call_details(proposal: &Value) -> Option<(String, Value)> {
        let function_call = proposal.get("kind")?.get("FunctionCall")?;

        Some((
            function_call.get("receiver_id")?.as_str()?.to_string(),
            function_call.get("actions")?.clone(),
        ))
    }
}
