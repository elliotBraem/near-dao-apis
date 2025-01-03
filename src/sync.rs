use crate::{
    bigquery::BigQueryClient,
    models::{Dao, Member, Proposal, Vote},
};
use sqlx::PgPool;
use std::error::Error;
use tracing::{error, info};

pub struct SyncService {
    db: PgPool,
    bq: BigQueryClient,
}

impl SyncService {
    pub fn new(db: PgPool, bq: BigQueryClient) -> Self {
        Self { db, bq }
    }

    pub async fn initial_sync(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting initial sync");

        // 1. Fetch all DAO accounts
        let dao_accounts = self.bq.fetch_dao_accounts().await?;

        for account in dao_accounts {
            // Insert DAO
            let dao_id = self.insert_dao(&account).await?;

            // Fetch and process all historical proposals
            self.sync_dao_data(&account, dao_id, None).await?;
        }

        info!("Initial sync completed");
        Ok(())
    }

    pub async fn incremental_sync(&self, start_block: i64) -> Result<(), Box<dyn Error>> {
        info!("Starting incremental sync from block {}", start_block);

        // Get all existing DAOs
        let daos: Vec<(i32, String)> =
            sqlx::query_as!((i32, String), "SELECT id, account_id FROM daos")
                .fetch_all(&self.db)
                .await?;

        // Update each DAO's data
        for (dao_id, account_id) in daos {
            self.sync_dao_data(&account_id, dao_id, Some(start_block))
                .await?;
        }

        info!("Incremental sync completed");
        Ok(())
    }

    async fn sync_dao_data(
        &self,
        dao_account: &str,
        dao_id: i32,
        start_block: Option<i64>,
    ) -> Result<(), Box<dyn Error>> {
        info!("Syncing data for DAO: {}", dao_account);

        // Fetch proposals
        let proposals = self
            .bq
            .fetch_dao_proposals(dao_account, start_block)
            .await?;

        for action in proposals {
            if let Err(e) = self.process_proposal(dao_id, &action).await {
                error!("Error processing proposal: {}", e);
            }
        }

        // Fetch votes
        let votes = self
            .bq
            .fetch_proposal_votes(dao_account, start_block)
            .await?;

        for action in votes {
            if let Err(e) = self.process_vote(dao_id, &action).await {
                error!("Error processing vote: {}", e);
            }
        }

        Ok(())
    }

    async fn insert_dao(&self, account_id: &str) -> Result<i32, Box<dyn Error>> {
        let dao = sqlx::query_as!(
            Dao,
            r#"
            INSERT INTO daos (account_id)
            VALUES ($1)
            ON CONFLICT (account_id) DO UPDATE
                SET account_id = EXCLUDED.account_id
            RETURNING id, account_id, created_at
            "#,
            account_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(dao.id)
    }

    async fn process_proposal(
        &self,
        dao_id: i32,
        action: &BigQueryAction,
    ) -> Result<(), Box<dyn Error>> {
        let args: serde_json::Value = serde_json::from_str(&action.args.to_string())?;

        // Extract proposal details
        let proposal_kind = self.bq.parse_proposal_kind(&args)?;

        // Process based on proposal type
        match proposal_kind.as_str() {
            "AddMember" | "RemoveMember" => {
                if let Some((member_id, role)) = self.bq.extract_member_details(&args) {
                    self.handle_member_proposal(
                        dao_id,
                        &action.signer_account_id,
                        &proposal_kind,
                        &member_id,
                        &role,
                    )
                    .await?;
                }
            }
            "Transfer" => {
                if let Some((token_id, receiver_id, amount, msg)) =
                    self.bq.extract_transfer_details(&args)
                {
                    self.handle_transfer_proposal(
                        dao_id,
                        &action.signer_account_id,
                        &token_id,
                        &receiver_id,
                        &amount,
                        msg.as_deref(),
                    )
                    .await?;
                }
            }
            "FunctionCall" => {
                if let Some((receiver_id, actions)) = self.bq.extract_function_call_details(&args) {
                    self.handle_function_call_proposal(
                        dao_id,
                        &action.signer_account_id,
                        &receiver_id,
                        &actions,
                    )
                    .await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn process_vote(
        &self,
        dao_id: i32,
        action: &BigQueryAction,
    ) -> Result<(), Box<dyn Error>> {
        let args: serde_json::Value = serde_json::from_str(&action.args.to_string())?;

        // Extract vote details and update the database
        // TODO: Implement vote processing

        Ok(())
    }

    // Handler functions for different proposal types
    async fn handle_member_proposal(
        &self,
        dao_id: i32,
        proposer: &str,
        proposal_type: &str,
        member_id: &str,
        role: &str,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: Implement member proposal handling
        Ok(())
    }

    async fn handle_transfer_proposal(
        &self,
        dao_id: i32,
        proposer: &str,
        token_id: &str,
        receiver_id: &str,
        amount: &str,
        msg: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: Implement transfer proposal handling
        Ok(())
    }

    async fn handle_function_call_proposal(
        &self,
        dao_id: i32,
        proposer: &str,
        receiver_id: &str,
        actions: &serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: Implement function call proposal handling
        Ok(())
    }
}
