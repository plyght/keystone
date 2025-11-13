# Birch

Peel. Rotate. Renew.

Birch is an open-source CLI tool for safe, fast secret rotation. It updates local `.env` files and production host secrets by name, without proxying traffic. Your applications call provider APIs directly with their own keys.

## Features

- **Key Pools**: Pre-configure multiple API keys for automatic sequential rotation on rate limits
- **Dev Mode**: Update `.env` files atomically with rollback support
- **Production Mode**: Integrate with major hosting providers (Vercel, Netlify, Render, Cloudflare, Fly.io) and cloud secret managers (AWS Secrets Manager, GCP Secret Manager, Azure Key Vault)
- **App-Signal Rotation**: Accept rotation requests from applications on rate limits or other triggers
- **Manual and Scheduled**: Operator-triggered and cron-friendly
- **Rollback**: Time-boxed rollback with automatic key revocation
- **Audit Logging**: Cryptographically signed logs with Ed25519
- **Safety Rails**: Single-writer locks, cooldowns, dry-run mode, maintenance windows

## Installation

```bash
cargo install --path .
```

Or download pre-built binaries from releases.

## Quick Start

### Initialize Configuration

```bash
birch config init
```

This creates `~/.birch/config.toml` with default settings.

### Dev Mode: Rotate a Secret in .env

```bash
birch rotate MY_API_KEY --env dev
```

This updates `MY_API_KEY` in your `.env` file and saves a rollback copy to `.birch-rollback`.

### Production Mode: Rotate a Secret on Vercel

```bash
export VERCEL_TOKEN="your-token"
export VERCEL_PROJECT_ID="your-project-id"

birch rotate MY_API_KEY --env prod --service vercel --redeploy
```

This updates the secret in Vercel and optionally triggers a redeploy.

### Start the Daemon for App-Signal Rotation

```bash
birch daemon start
```

The daemon listens on `127.0.0.1:9123` for rotation signals from your application:

```bash
curl -X POST http://127.0.0.1:9123/rotate \
  -H "Content-Type: application/json" \
  -d '{"secret_name": "MY_API_KEY", "env": "prod", "service": "vercel"}'
```

### Rollback a Secret

```bash
birch rollback MY_API_KEY --env prod --service vercel
```

### View Audit Logs

```bash
birch audit MY_API_KEY --env prod
```

### Key Pools for Automatic Rotation

Set up a pool of API keys for automatic rotation when rate limits are hit:

```bash
# Create a pool with multiple keys
birch pool init TIKTOK_API_KEY --keys "sk_key1,sk_key2,sk_key3"

# Check pool status
birch pool status TIKTOK_API_KEY

# Rotate (automatically uses next available key from pool)
birch rotate TIKTOK_API_KEY --env prod --service vercel
```

When your app hits a rate limit (HTTP 429), it can trigger automatic rotation:

```javascript
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
}
```

See [Key Pool Documentation](./docs/content/docs/usage/key-pools.mdx) for details.

### Zero-Config SDK

For even simpler integration, use the `@birch/client` SDK that automatically handles rate limits:

```bash
npm install @birch/client
```

```typescript
import '@birch/client/auto';

const response = await fetch('https://api.tiktok.com/v1/videos', {
  headers: {
    Authorization: `Bearer ${process.env.TIKTOK_API_KEY}`
  }
});
```

That's it! The SDK automatically:
- Detects which API keys are being used
- Intercepts 429 responses
- Rotates to the next key in the pool
- Retries the request immediately

Works with Next.js, Express, vanilla Node.js, and any framework. See [SDK Documentation](./docs/content/docs/sdk.mdx) for details.

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
- [Operator Runbook](./docs/content/docs/operators/runbook.mdx)
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

