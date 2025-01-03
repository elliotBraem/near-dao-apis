use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

// Response types
#[derive(Serialize)]
pub struct DaoResponse {
    id: i32,
    account_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    proposal_count: i64,
    member_count: i64,
}

#[derive(Serialize)]
pub struct ProposalResponse {
    id: i32,
    proposal_id: i64,
    proposer: String,
    proposal_type: String,
    description: String,
    status: String,
    target_member: Option<String>,
    target_role: Option<String>,
    receiver_id: Option<String>,
    actions: Option<serde_json::Value>,
    token_id: Option<String>,
    amount: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    votes: Vec<VoteInfo>,
}

#[derive(Serialize)]
pub struct VoteInfo {
    voter: String,
    vote: String,
    voted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct MemberResponse {
    account_id: String,
    roles: Vec<String>,
    added_at: chrono::DateTime<chrono::Utc>,
}

// Query parameters
#[derive(Deserialize)]
pub struct PaginationParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub fn create_router(db: PgPool) -> Router {
    Router::new()
        .route("/daos", get(list_daos))
        .route("/daos/:dao_id", get(get_dao))
        .route("/daos/:dao_id/proposals", get(list_proposals))
        .route("/daos/:dao_id/proposals/:proposal_id", get(get_proposal))
        .route("/daos/:dao_id/members", get(list_members))
        .with_state(Arc::new(db))
}

// Route handlers
async fn list_daos(
    State(db): State<Arc<PgPool>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<DaoResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    let daos = sqlx::query_as!(
        DaoResponse,
        r#"
        SELECT 
            d.id,
            d.account_id,
            d.created_at,
            COUNT(DISTINCT p.id) as proposal_count,
            COUNT(DISTINCT m.id) as member_count
        FROM daos d
        LEFT JOIN proposals p ON p.dao_id = d.id
        LEFT JOIN members m ON m.dao_id = d.id
        GROUP BY d.id
        ORDER BY d.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&*db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(daos))
}

async fn get_dao(
    State(db): State<Arc<PgPool>>,
    Path(dao_id): Path<i32>,
) -> Result<Json<DaoResponse>, StatusCode> {
    let dao = sqlx::query_as!(
        DaoResponse,
        r#"
        SELECT 
            d.id,
            d.account_id,
            d.created_at,
            COUNT(DISTINCT p.id) as proposal_count,
            COUNT(DISTINCT m.id) as member_count
        FROM daos d
        LEFT JOIN proposals p ON p.dao_id = d.id
        LEFT JOIN members m ON m.dao_id = d.id
        WHERE d.id = $1
        GROUP BY d.id
        "#,
        dao_id
    )
    .fetch_one(&*db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(dao))
}

async fn list_proposals(
    State(db): State<Arc<PgPool>>,
    Path(dao_id): Path<i32>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<ProposalResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    let proposals = sqlx::query_as!(
        ProposalResponse,
        r#"
        SELECT 
            p.id,
            p.proposal_id,
            p.proposer,
            p.proposal_type,
            p.description,
            p.status,
            p.target_member,
            p.target_role,
            p.receiver_id,
            p.actions,
            p.token_id,
            p.amount,
            p.created_at,
            COALESCE(
                json_agg(
                    json_build_object(
                        'voter', v.voter,
                        'vote', v.vote,
                        'voted_at', v.voted_at
                    )
                ) FILTER (WHERE v.id IS NOT NULL),
                '[]'
            ) as votes
        FROM proposals p
        LEFT JOIN votes v ON v.proposal_id = p.id
        WHERE p.dao_id = $1
        GROUP BY p.id
        ORDER BY p.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        dao_id,
        limit,
        offset
    )
    .fetch_all(&*db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(proposals))
}

async fn get_proposal(
    State(db): State<Arc<PgPool>>,
    Path((dao_id, proposal_id)): Path<(i32, i32)>,
) -> Result<Json<ProposalResponse>, StatusCode> {
    let proposal = sqlx::query_as!(
        ProposalResponse,
        r#"
        SELECT 
            p.id,
            p.proposal_id,
            p.proposer,
            p.proposal_type,
            p.description,
            p.status,
            p.target_member,
            p.target_role,
            p.receiver_id,
            p.actions,
            p.token_id,
            p.amount,
            p.created_at,
            COALESCE(
                json_agg(
                    json_build_object(
                        'voter', v.voter,
                        'vote', v.vote,
                        'voted_at', v.voted_at
                    )
                ) FILTER (WHERE v.id IS NOT NULL),
                '[]'
            ) as votes
        FROM proposals p
        LEFT JOIN votes v ON v.proposal_id = p.id
        WHERE p.dao_id = $1 AND p.id = $2
        GROUP BY p.id
        "#,
        dao_id,
        proposal_id
    )
    .fetch_one(&*db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(proposal))
}

async fn list_members(
    State(db): State<Arc<PgPool>>,
    Path(dao_id): Path<i32>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<MemberResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    let members = sqlx::query_as!(
        MemberResponse,
        r#"
        SELECT 
            account_id,
            roles,
            added_at
        FROM members
        WHERE dao_id = $1
        ORDER BY added_at DESC
        LIMIT $2 OFFSET $3
        "#,
        dao_id,
        limit,
        offset
    )
    .fetch_all(&*db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(members))
}
