mod handlers;

use crate::{
    bigquery::BigQueryClient,
    models::{Dao, Member, Proposal, Vote},
};
use handlers::*;
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
        action: &crate::models::BigQueryAction,
    ) -> Result<(), Box<dyn Error>> {
        let args: serde_json::Value = serde_json::from_str(&action.args.to_string())?;
        let proposal_id = args
            .get("proposal_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing proposal_id")?;

        // Extract proposal details
        let proposal_kind = self.bq.parse_proposal_kind(&args)?;

        // Process based on proposal type
        match proposal_kind.as_str() {
            "AddMember" | "RemoveMember" => {
                if let Some((member_id, role)) = self.bq.extract_member_details(&args)? {
                    handle_member_proposal(
                        &self.db,
                        dao_id,
                        &action.signer_account_id,
                        &proposal_kind,
                        &member_id,
                        &role,
                        proposal_id,
                    )
                    .await?;
                }
            }
            "Transfer" => {
                if let Some((token_id, receiver_id, amount, msg)) =
                    self.bq.extract_transfer_details(&args)?
                {
                    handle_transfer_proposal(
                        &self.db,
                        dao_id,
                        &action.signer_account_id,
                        &token_id,
                        &receiver_id,
                        &amount,
                        msg.as_deref(),
                        proposal_id,
                    )
                    .await?;
                }
            }
            "FunctionCall" => {
                if let Some((receiver_id, actions)) =
                    self.bq.extract_function_call_details(&args)?
                {
                    handle_function_call_proposal(
                        &self.db,
                        dao_id,
                        &action.signer_account_id,
                        &receiver_id,
                        &actions,
                        proposal_id,
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
        action: &crate::models::BigQueryAction,
    ) -> Result<(), Box<dyn Error>> {
        let args: serde_json::Value = serde_json::from_str(&action.args.to_string())?;

        // Extract vote details
        let proposal_id = args
            .get("proposal_id")
            .and_then(|v| v.as_i64())
            .ok_or("Missing proposal_id")?;
        let vote = args
            .get("vote")
            .and_then(|v| v.as_str())
            .ok_or("Missing vote")?;

        // Get the internal proposal ID
        let internal_proposal_id = sqlx::query!(
            "SELECT id FROM proposals WHERE dao_id = $1 AND proposal_id = $2",
            dao_id,
            proposal_id
        )
        .fetch_one(&self.db)
        .await?
        .id;

        // Process the vote
        handle_vote(
            &self.db,
            internal_proposal_id,
            &action.signer_account_id,
            vote,
        )
        .await?;

        Ok(())
    }
}
