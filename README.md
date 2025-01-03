# NEAR DAO APIs

A Rust-based service that syncs and serves DAO data from NEAR Protocol, providing REST APIs for accessing DAO information, proposals, and member data.

## Architecture

### Data Flow
```
BigQuery (NEAR Data) → Sync Service → PostgreSQL → REST APIs
```

### Components

1. **BigQuery Integration** (`src/bigquery/`)
   - Fetches DAO-related data from NEAR Protocol's BigQuery dataset
   - Handles incremental syncing using block timestamps
   - Parses complex proposal data into structured formats

2. **Data Models** (`src/models.rs`)
   - DAO entities and their relationships
   - Proposal types:
     - AddMember/RemoveMember: Member role management
     - FunctionCall: Smart contract interactions
     - Transfer: Token transfers
     - Vote: Signaling proposals
   - Member management with roles
   - Vote tracking

3. **Sync Service** (`src/sync/`)
   - Initial historical data sync
   - Real-time updates (1-minute intervals)
   - Error handling and retry logic
   - Transaction processing and data consistency

4. **REST APIs** (`src/routes/`)
   - `GET /daos` - List all DAOs
   - `GET /daos/:id` - Get DAO details
   - `GET /daos/:id/proposals` - List DAO proposals
   - `GET /daos/:id/proposals/:id` - Get proposal details with votes
   - `GET /daos/:id/members` - List DAO members

### Database Schema

```sql
daos
  - id (PK)
  - account_id (unique)
  - created_at

proposals
  - id (PK)
  - dao_id (FK)
  - proposal_id
  - proposer
  - proposal_type
  - description
  - status
  - target_member
  - target_role
  - receiver_id
  - actions
  - token_id
  - amount
  - created_at
  - updated_at

members
  - id (PK)
  - dao_id (FK)
  - account_id
  - roles
  - added_at

votes
  - id (PK)
  - proposal_id (FK)
  - voter
  - vote
  - voted_at
```

## Setup

### Prerequisites
- Rust (latest stable)
- PostgreSQL
- BigQuery access and credentials
- NEAR Protocol BigQuery dataset access

### Environment Variables
Create a `.env` file with:
```env
# Database
DATABASE_URL=postgresql://localhost/dao_apis

# BigQuery
BIGQUERY_PROJECT_ID=your-project-id
BIGQUERY_CREDENTIALS_PATH=/path/to/credentials.json

# Logging
RUST_LOG=info

# Server
PORT=3000
HOST=127.0.0.1
```

### Database Setup
```bash
# Create database
createdb dao_apis

# Run migrations (automatic on startup)
# Migrations are in migrations/20240118000000_initial_schema.sql
```

### Running the Service
```bash
# Build and run
cargo run

# The service will:
# 1. Run database migrations
# 2. Perform initial data sync
# 3. Start real-time sync (every minute)
# 4. Start HTTP server on PORT (default: 3000)
```

## API Usage

### List DAOs
```bash
curl "http://localhost:3000/daos?limit=10&offset=0"
```

### Get DAO Details
```bash
curl "http://localhost:3000/daos/1"
```

### List DAO Proposals
```bash
curl "http://localhost:3000/daos/1/proposals?limit=10&offset=0"
```

### Get Proposal Details
```bash
curl "http://localhost:3000/daos/1/proposals/1"
```

### List DAO Members
```bash
curl "http://localhost:3000/daos/1/members?limit=10&offset=0"
```

## Development

### Project Structure
```
src/
├── bigquery/           # BigQuery integration
│   ├── mod.rs         # Module definitions
│   ├── client.rs      # BigQuery client
│   └── queries.rs     # SQL queries
├── sync/              # Data sync service
│   ├── mod.rs         # Sync service implementation
│   └── handlers.rs    # Proposal/vote handlers
├── models.rs          # Data models
├── routes/            # API routes
│   └── mod.rs         # Route handlers
└── main.rs            # Application entry point

migrations/           # Database migrations
└── 20240118000000_initial_schema.sql
```

### Adding New Features
1. Update models in `models.rs`
2. Add migrations in `migrations/`
3. Implement data sync in `sync/`
4. Add API endpoints in `routes/`

### Testing
```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo test
