-- Create DAOs table
CREATE TABLE daos (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create proposals table
CREATE TABLE proposals (
    id SERIAL PRIMARY KEY,
    dao_id INTEGER REFERENCES daos(id),
    proposal_id BIGINT NOT NULL,  -- Original proposal ID from contract
    proposer VARCHAR NOT NULL,
    proposal_type VARCHAR NOT NULL,  -- 'AddMember', 'RemoveMember', 'FunctionCall', 'Transfer', 'Vote'
    description TEXT NOT NULL,
    status VARCHAR NOT NULL,  -- 'InProgress', 'Approved', 'Rejected', 'Removed'
    
    -- Fields for different proposal types
    target_member VARCHAR,  -- For AddMember/RemoveMember
    target_role VARCHAR,   -- For AddMember/RemoveMember
    receiver_id VARCHAR,   -- For FunctionCall/Transfer
    actions JSONB,         -- For FunctionCall details
    token_id VARCHAR,      -- For Transfer
    amount VARCHAR,        -- For Transfer (stored as string to handle U128)
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(dao_id, proposal_id)
);

-- Create members table
CREATE TABLE members (
    id SERIAL PRIMARY KEY,
    dao_id INTEGER REFERENCES daos(id),
    account_id VARCHAR NOT NULL,
    roles VARCHAR[] NOT NULL DEFAULT '{}',  -- Array of role names
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dao_id, account_id)
);

-- Create votes table
CREATE TABLE votes (
    id SERIAL PRIMARY KEY,
    proposal_id INTEGER REFERENCES proposals(id),
    voter VARCHAR NOT NULL,
    vote VARCHAR NOT NULL,  -- 'Approve', 'Reject', 'Remove'
    voted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(proposal_id, voter)
);

-- Create indexes for common queries
CREATE INDEX idx_daos_account_id ON daos(account_id);
CREATE INDEX idx_proposals_dao_id_status ON proposals(dao_id, status);
CREATE INDEX idx_proposals_proposal_id ON proposals(proposal_id);
CREATE INDEX idx_proposals_type_status ON proposals(proposal_type, status);
CREATE INDEX idx_members_dao_account ON members(dao_id, account_id);
CREATE INDEX idx_votes_proposal_voter ON votes(proposal_id, voter);
