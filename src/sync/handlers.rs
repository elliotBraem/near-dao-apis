use crate::models::{Proposal, Vote};
use sqlx::PgPool;
use std::error::Error;
use tracing::{error, info};

pub(crate) async fn handle_member_proposal(
    db: &PgPool,
    dao_id: i32,
    proposer: &str,
    proposal_type: &str,
    member_id: &str,
    role: &str,
    proposal_id: i64,
) -> Result<(), Box<dyn Error>> {
    // Insert the proposal
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        INSERT INTO proposals (
            dao_id, proposal_id, proposer, proposal_type,
            description, status, target_member, target_role
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (dao_id, proposal_id) DO UPDATE
        SET status = EXCLUDED.status
        RETURNING *
        "#,
        dao_id,
        proposal_id,
        proposer,
        proposal_type,
        format!("{} {} as {}", proposal_type, member_id, role),
        "InProgress",
        Some(member_id),
        Some(role)
    )
    .fetch_one(db)
    .await?;

    info!(
        "Processed member proposal {} for DAO {}: {} {}",
        proposal.id, dao_id, member_id, role
    );

    Ok(())
}

pub(crate) async fn handle_transfer_proposal(
    db: &PgPool,
    dao_id: i32,
    proposer: &str,
    token_id: &str,
    receiver_id: &str,
    amount: &str,
    msg: Option<&str>,
    proposal_id: i64,
) -> Result<(), Box<dyn Error>> {
    let description = match msg {
        Some(m) => format!(
            "Transfer {} {} to {} ({})",
            amount, token_id, receiver_id, m
        ),
        None => format!("Transfer {} {} to {}", amount, token_id, receiver_id),
    };

    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        INSERT INTO proposals (
            dao_id, proposal_id, proposer, proposal_type,
            description, status, token_id, receiver_id, amount
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (dao_id, proposal_id) DO UPDATE
        SET status = EXCLUDED.status
        RETURNING *
        "#,
        dao_id,
        proposal_id,
        proposer,
        "Transfer",
        description,
        "InProgress",
        Some(token_id),
        Some(receiver_id),
        Some(amount)
    )
    .fetch_one(db)
    .await?;

    info!(
        "Processed transfer proposal {} for DAO {}: {} {} to {}",
        proposal.id, dao_id, amount, token_id, receiver_id
    );

    Ok(())
}

pub(crate) async fn handle_function_call_proposal(
    db: &PgPool,
    dao_id: i32,
    proposer: &str,
    receiver_id: &str,
    actions: &serde_json::Value,
    proposal_id: i64,
) -> Result<(), Box<dyn Error>> {
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        INSERT INTO proposals (
            dao_id, proposal_id, proposer, proposal_type,
            description, status, receiver_id, actions
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (dao_id, proposal_id) DO UPDATE
        SET status = EXCLUDED.status
        RETURNING *
        "#,
        dao_id,
        proposal_id,
        proposer,
        "FunctionCall",
        format!("Function call to {}", receiver_id),
        "InProgress",
        Some(receiver_id),
        actions as _
    )
    .fetch_one(db)
    .await?;

    info!(
        "Processed function call proposal {} for DAO {}: call to {}",
        proposal.id, dao_id, receiver_id
    );

    Ok(())
}

pub(crate) async fn handle_vote(
    db: &PgPool,
    proposal_id: i32,
    voter: &str,
    vote: &str,
) -> Result<(), Box<dyn Error>> {
    // Insert the vote
    let vote_record = sqlx::query_as!(
        Vote,
        r#"
        INSERT INTO votes (proposal_id, voter, vote)
        VALUES ($1, $2, $3)
        ON CONFLICT (proposal_id, voter) DO UPDATE
        SET vote = EXCLUDED.vote
        RETURNING *
        "#,
        proposal_id,
        voter,
        vote
    )
    .fetch_one(db)
    .await?;

    info!(
        "Processed vote for proposal {}: {} voted {}",
        proposal_id, voter, vote
    );

    // Update proposal status based on vote count
    // TODO: Implement vote counting logic to determine proposal status

    Ok(())
}
