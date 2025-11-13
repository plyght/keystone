# Keystone API

Backend API service for Keystone SaaS, built with Rust and Axum.

## Architecture

- **Framework**: Axum (async Rust web framework)
- **Database**: PostgreSQL via Supabase
- **Cache**: Redis
- **Auth**: JWT validation + API keys
- **Encryption**: ChaCha20Poly1305 envelope encryption

## Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL (or Supabase account)
- Redis

### Environment Variables

Copy `../../env.example` to `.env` and configure:

```bash
# Supabase
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Database
DATABASE_URL=postgresql://postgres:password@localhost:54322/postgres

# Redis
REDIS_URL=redis://localhost:6379

# API
API_HOST=0.0.0.0
API_PORT=3000

# Encryption
VAULT_MASTER_KEY=your-64-char-hex-key

# Logging
RUST_LOG=info
```

### Run Migrations

```bash
cd ../../supabase
supabase migration up
```

### Start the Server

```bash
cargo run --bin keystone-api
```

The API will be available at `http://localhost:3000`.

## API Endpoints

### Health Check

```
GET /health
```

### Workspaces

```
POST   /api/v1/workspaces
GET    /api/v1/workspaces
GET    /api/v1/workspaces/:id
PUT    /api/v1/workspaces/:id
DELETE /api/v1/workspaces/:id
```

### Members

```
POST   /api/v1/workspaces/:id/members
GET    /api/v1/workspaces/:id/members
PUT    /api/v1/workspaces/:id/members/:user_id
DELETE /api/v1/workspaces/:id/members/:user_id
```

### Providers

```
POST   /api/v1/workspaces/:id/providers
GET    /api/v1/workspaces/:id/providers
GET    /api/v1/workspaces/:id/providers/:provider
PUT    /api/v1/workspaces/:id/providers/:provider
DELETE /api/v1/workspaces/:id/providers/:provider
```

### Credentials

```
POST /api/v1/workspaces/:id/credentials
GET  /api/v1/workspaces/:id/credentials/:provider/:secret_name
```

### API Keys

```
POST   /api/v1/workspaces/:id/api-keys
GET    /api/v1/workspaces/:id/api-keys
DELETE /api/v1/workspaces/:id/api-keys/:key_id
```

## Authentication

All requests require authentication via Bearer token:

```bash
curl -H "Authorization: Bearer <your-api-key>" \
  https://api.keystone.birch.sh/api/v1/workspaces
```

## Testing

Run tests:

```bash
cargo test
```

## Project Structure

```
src/
├── main.rs                 # Entry point
├── lib.rs                  # Module exports
├── supabase/              # Database client
├── auth/                  # Authentication & authorization
├── vault/                 # Credential encryption & storage
├── credentials/           # Credential resolution & caching
├── workspace/             # Multi-tenancy & RBAC
├── metering/              # Usage tracking
└── api/                   # HTTP routes & handlers
    ├── routes.rs
    └── handlers/
        ├── workspaces.rs
        ├── members.rs
        ├── providers.rs
        ├── credentials.rs
        └── api_keys.rs
```

## Security

### Encryption

- **Algorithm**: ChaCha20Poly1305 (AEAD)
- **Key Derivation**: Workspace-specific keys derived from master key
- **Nonce**: Random 12-byte nonce per encryption operation

### Database Security

- **RLS**: Row Level Security enforces workspace isolation
- **Prepared Statements**: All queries use prepared statements
- **Connection Pooling**: Deadpool for safe connection management

### Authentication

- **JWT**: Supabase Auth tokens for user sessions
- **API Keys**: Argon2 hashed keys for programmatic access
- **RBAC**: Role-based access control with 5 roles

## Deployment

### Docker

```bash
docker build -t keystone-api .
docker run -p 3000:3000 --env-file .env keystone-api
```

### Fly.io

```bash
fly launch
fly secrets set VAULT_MASTER_KEY=...
fly deploy
```

## License

MIT - See LICENSE file in repository root

