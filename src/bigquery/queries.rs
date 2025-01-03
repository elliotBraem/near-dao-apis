// Query to find all DAO accounts
pub(crate) const QUERY_DAO_ACCOUNTS: &str = r#"
SELECT DISTINCT account_id
FROM near_mainnet.accounts
WHERE account_id LIKE '%.sputnik-dao.near'
AND action_kind = 'CREATE_ACCOUNT'
ORDER BY block_timestamp ASC
"#;

// Query to find all proposals for a DAO
pub(crate) fn query_dao_proposals(dao_id: &str, start_block: Option<i64>) -> String {
    let block_filter = start_block
        .map(|block| format!("AND block_timestamp >= {}", block))
        .unwrap_or_default();

    format!(
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
    )
}

// Query to find all votes for a DAO
pub(crate) fn query_dao_votes(dao_id: &str, start_block: Option<i64>) -> String {
    let block_filter = start_block
        .map(|block| format!("AND block_timestamp >= {}", block))
        .unwrap_or_default();

    format!(
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
    )
}
