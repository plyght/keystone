# Birch

Peel. Rotate. Renew.

Birch is a minimal, open-source key rotation engine for modern API-driven applications. Rotate secrets, manage key pools for rate-limited APIs, sign audit logs, and update environments—all from a single Rust binary. No Vault required. No cloud lock-in. Works standalone or integrates with your existing infrastructure.

## Why Birch

Most teams handle secret rotation with ad-hoc scripts, manual processes, or heavyweight tools like Vault. Birch solves the 90% use case: rotating API keys for rate-limited services, updating `.env` files and production secrets, and maintaining audit trails—without the operational overhead.

**Key Pools** are Birch's differentiating feature. When your application hits a rate limit (HTTP 429), Birch automatically rotates to the next key in your pre-configured pool, marks the exhausted key for rotation, and continues serving traffic. This solves the common problem of managing multiple Stripe keys, OpenAI keys, or SendGrid keys without manual intervention.

## Comparison

| Feature | Birch | Doppler | Vault | Infisical |
|---------|-------|---------|-------|-----------|
| Rotate API keys | Yes | Yes | Script only | Yes |
| Key pool with rate-limit cycling | Yes | No | No | Partial |
| Signed audit logs | Yes | No | No | No |
| Standalone operation | Yes | No | Yes | No |
| Host integrations | Yes | Yes | Script only | Yes |
| Zero SaaS dependency | Yes | No | Yes | No |

## Installation

### Binary

Download pre-built binaries from [releases](https://github.com/plyght/birch/releases).

### From Source

```bash
cargo install birch
```

Or build from source:

```bash
git clone https://github.com/plyght/birch.git
cd birch
cargo build --release
sudo cp target/release/birch /usr/local/bin/
```

## Quick Start

### Initialize Configuration

```bash
birch config init
```

This creates `~/.birch/config.toml` with default settings.

## Hero Examples

### Example 1: Simple Rotation

Rotate your OpenAI key every week and update your `.env` file:

```bash
# Dev environment
birch rotate OPENAI_API_KEY --env dev

# Production (Vercel)
export VERCEL_TOKEN="your-token"
export VERCEL_PROJECT_ID="your-project-id"
birch rotate OPENAI_API_KEY --env prod --service vercel --redeploy
```

Schedule with cron:

```bash
0 2 * * 0 birch rotate OPENAI_API_KEY --env prod --service vercel
```

### Example 2: Rate-Limit Pool Cycling

Use a pool of 5 API keys. On HTTP 429, automatically advance to the next key and rotate the exhausted key:

**Step 1: Create the pool**

```bash
birch pool init TIKTOK_API_KEY \
  --keys "sk_key1,sk_key2,sk_key3,sk_key4,sk_key5"
```

**Step 2: Start the daemon**

```bash
birch daemon start
```

**Step 3: Integrate with your application**

```javascript
// Your app detects 429
if (response.status === 429) {
  await fetch('http://localhost:9123/rotate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      secret_name: 'TIKTOK_API_KEY',
      env: 'prod',
      service: 'vercel'
    })
  });
  // Retry request with new key
}
```

**Step 4: Use the SDK for zero-config integration**

```bash
npm install @inaplight/birch-client
```

```typescript
import '@inaplight/birch-client/auto';

// SDK automatically detects 429s, rotates keys, and retries
const response = await fetch('https://api.tiktok.com/v1/videos', {
  headers: {
    Authorization: `Bearer ${process.env.TIKTOK_API_KEY}`
  }
});
```

### Example 3: Host Integration with Rollback

Rotate production key on Fly.io, update secret store, trigger redeploy, with automatic rollback support:

```bash
export FLY_API_TOKEN="your-token"
export FLY_APP_NAME="your-app"

# Dry-run first
birch rotate STRIPE_SECRET_KEY --env prod --service fly --dry-run

# Execute rotation
birch rotate STRIPE_SECRET_KEY --env prod --service fly --redeploy

# If something breaks, rollback within the window (default: 1 hour)
birch rollback STRIPE_SECRET_KEY --env prod --service fly --redeploy
```

## Features

### Key Pools

Pre-configure multiple API keys for automatic sequential rotation. When a rate limit is detected, Birch switches to the next available key, marks the exhausted key for rotation, and continues serving traffic. Ideal for TikTok, Twitter, Stripe, OpenAI, SendGrid, and other rate-limited APIs.

### Dev Mode

Update `.env` files atomically with rollback support. Perfect for local development and testing.

### Production Mode

Integrate with major hosting providers (Vercel, Netlify, Render, Cloudflare Workers, Fly.io) and cloud secret managers (AWS Secrets Manager, GCP Secret Manager, Azure Key Vault). Update secrets and optionally trigger redeployments.

### App-Signal Rotation

Accept rotation requests from applications on rate limits or other triggers. Run the daemon and let your application signal rotations as needed.

### Manual and Scheduled

Operator-triggered rotations and cron-friendly commands. Perfect for scheduled rotations and manual interventions.

### Rollback

Time-boxed rollback with automatic key revocation. Rollback within the configured window (default: 1 hour) to restore previous secrets.

### Audit Logging

Cryptographically signed logs with Ed25519. Every rotation is logged with timestamp, actor, action, and signature for compliance and forensics.

### Interactive Dashboard

Real-time TUI dashboard for monitoring operations. View daemon status, audit logs, pool health, and rotation metrics in a clean terminal interface. Navigate with keyboard shortcuts, auto-refreshes every 5 seconds.

### Safety Rails

Single-writer locks prevent concurrent rotations. Cooldowns prevent rapid successive rotations. Dry-run mode previews changes. Maintenance windows restrict production changes to specific times.

## Configuration

Edit `~/.birch/config.toml`:

```toml
audit_log_path = "/Users/you/.birch/logs"
cooldown_seconds = 60
rollback_window_seconds = 3600
daemon_bind = "127.0.0.1:9123"
pool_low_threshold = 2

[[maintenance_windows]]
start_hour = 2
end_hour = 6
days = ["Saturday", "Sunday"]

[connector_auth]
vercel_token = "optional-token-here"
```

Environment variables override config file settings:

- `BIRCH_AUDIT_LOG_PATH`
- `BIRCH_COOLDOWN_SECONDS`
- `BIRCH_ROLLBACK_WINDOW_SECONDS`
- `BIRCH_POOL_LOW_THRESHOLD`
- `VERCEL_TOKEN`, `NETLIFY_AUTH_TOKEN`, `RENDER_API_KEY`, etc.

## Supported Providers

### Hosting Providers

- **Vercel**: Requires `VERCEL_TOKEN` and `VERCEL_PROJECT_ID`
- **Netlify**: Requires `NETLIFY_AUTH_TOKEN` and `NETLIFY_SITE_ID`
- **Render**: Requires `RENDER_API_KEY` and `RENDER_SERVICE_ID`
- **Cloudflare Workers**: Requires `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`, and `CLOUDFLARE_WORKER_NAME`
- **Fly.io**: Requires `FLY_API_TOKEN` and `FLY_APP_NAME`

### Cloud Secret Managers

- **AWS Secrets Manager**: Requires `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_REGION`
- **GCP Secret Manager**: Requires `GOOGLE_APPLICATION_CREDENTIALS` and `GCP_PROJECT_ID`
- **Azure Key Vault**: Requires `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`, and `AZURE_VAULT_NAME`

Note: Cloud secret managers update secrets directly but do not automatically trigger application restarts. Restart your services manually or use the hosting provider connectors for automatic redeployment.

## Safety Features

- **Dry-Run Mode**: Use `--dry-run` to preview changes without applying them
- **Cooldown**: Prevents rapid successive rotations (default: 60 seconds)
- **Single-Writer Locks**: Per-secret/env locks prevent concurrent rotations
- **Maintenance Windows**: Time-based restrictions for production changes
- **Confirmation Prompts**: Explicit user confirmation for production operations
- **Masked Output**: Secrets are never printed in plaintext (shows `***` or last 4 chars)

## Documentation

Complete documentation is available in the `docs` directory, powered by Fumadocs.

Quick links:
- [Quick Start Guide](./docs/content/docs/quick-start.mdx)
- [Key Pools](./docs/content/docs/usage/key-pools.mdx)
- [Operator Runbook](./docs/content/docs/operators/runbook.mdx)
- [Invariants and Guarantees](./docs/content/docs/operators/invariants.mdx)
- [CLI Reference](./docs/content/docs/cli-reference.mdx)

To run the documentation locally:

```bash
cd docs
bun install
bun run dev
```

Then open http://localhost:3000

## Architecture

Birch is a single Rust binary with:
- CLI for manual operations
- Optional background daemon for app-signal handling
- File-based locking and audit logging
- Connector architecture for provider integrations

No traffic proxying. No central secret storage. Just direct updates to your `.env` files and provider APIs.

## License

MIT
