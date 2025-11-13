# Birch Examples

This directory contains example applications demonstrating how to use Birch for automatic API key rotation.

## Directory Structure

```
examples/
├── rust/          - Rust examples
└── typescript/    - TypeScript/Node.js examples
```

## Rust Examples

### App Signal Hook

Demonstrates how to manually send rotation signals to the Birch daemon from a Rust application.

**Location:** `rust/app_signal_hook.rs`

**Prerequisites:**
- Rust toolchain installed
- Birch daemon running (`birch daemon start`)

**Run:**
```bash
cd /path/to/birch
cargo run --example app_signal_hook
```

**What it does:**
- Simulates detecting a rate limit (HTTP 429)
- Sends a rotation request to the Birch daemon
- Displays the daemon's response

## TypeScript Examples

All TypeScript examples use the `@birch/client` SDK for automatic rotation.

### CLI Script

A standalone CLI script that makes API calls with automatic rotation.

**Location:** `typescript/cli-script/`

**Prerequisites:**
- Bun installed (or Node.js with npm)
- Birch daemon running
- API keys in environment variables

**Setup:**
```bash
cd examples/typescript/cli-script
bun install @birch/client
```

**Run:**
```bash
export TIKTOK_API_KEY="your-key"
export OPENAI_API_KEY="your-key"
bun run script.ts
```

**What it does:**
- Imports `@birch/client/auto` for zero-config setup
- Makes fetch requests to TikTok and OpenAI APIs
- Automatically rotates keys on 429 responses

### Express API

A REST API server built with Express that automatically rotates API keys.

**Location:** `typescript/express-api/`

**Prerequisites:**
- Bun installed (or Node.js with npm)
- Birch daemon running
- API keys configured

**Setup:**
```bash
cd examples/typescript/express-api
bun install @birch/client express
bun install -D @types/express
```

**Run:**
```bash
export TWITTER_API_KEY="your-key"
export TIKTOK_API_KEY="your-key"
bun run server.ts
```

**Test:**
```bash
curl http://localhost:3000/tweets
curl http://localhost:3000/tiktok
```

**What it does:**
- Imports `@birch/client/auto` at the top of the file
- Provides REST endpoints that call external APIs
- Automatically handles rate limits and key rotation

### Next.js App

A Next.js App Router application with API routes that use automatic rotation.

**Location:** `typescript/nextjs-app/`

**Prerequisites:**
- Bun installed (or Node.js with npm)
- Birch daemon running
- API keys configured

**Setup:**
```bash
cd examples/typescript/nextjs-app
bun install @birch/client next react react-dom
```

**Create `.env.local`:**
```bash
TIKTOK_API_KEY=your-key
```

**Run:**
```bash
bun run next dev
```

**Test:**
```bash
curl http://localhost:3000/api/tiktok
```

**What it does:**
- Imports `@birch/client/auto` in the root layout
- Provides API routes that fetch from external APIs
- Automatically rotates keys when rate limits are hit

## Common Setup

### 1. Start the Birch Daemon

All examples require the Birch daemon to be running:

```bash
birch daemon start
```

Verify it's running:
```bash
birch daemon status
# or
curl http://localhost:9123/health
```

### 2. Set Up Key Pools

For automatic rotation to work, set up key pools:

```bash
birch pool init TIKTOK_API_KEY --keys "key1,key2,key3"
birch pool init TWITTER_API_KEY --keys "key1,key2,key3"
birch pool init OPENAI_API_KEY --keys "key1,key2,key3"
```

### 3. Configure Environment Variables

Each example needs the appropriate API keys set as environment variables:

```bash
export TIKTOK_API_KEY="your-first-key"
export TWITTER_API_KEY="your-first-key"
export OPENAI_API_KEY="your-first-key"
```

## How It Works

### Rust Examples (Manual)

The Rust example demonstrates the manual approach:
1. Application detects a condition requiring rotation (e.g., 429 response)
2. Application sends HTTP POST to `http://localhost:9123/rotate`
3. Daemon queues the rotation and returns immediately
4. Daemon updates the key asynchronously

### TypeScript Examples (Automatic)

The TypeScript examples use the SDK for zero-config automation:
1. Import `@birch/client/auto` at the entry point
2. SDK intercepts all HTTP requests (fetch, axios, etc.)
3. SDK detects 429 responses automatically
4. SDK calls the daemon and retries with a new key immediately
5. Daemon updates production secrets asynchronously

## Troubleshooting

### "Daemon not available"

Ensure the daemon is running:
```bash
birch daemon start
curl http://localhost:9123/health
```

### "Could not detect secret name"

Enable debug mode to see what the SDK detects:
```bash
export BIRCH_DEBUG=true
bun run script.ts
```

The SDK looks for tokens in environment variables ending with:
- `API_KEY`
- `TOKEN`
- `SECRET`

### "Pool exhausted"

Check your pool status:
```bash
birch pool status TIKTOK_API_KEY
```

Add more keys if needed:
```bash
birch pool add TIKTOK_API_KEY --key "new-key"
```

## Further Reading

- [Birch CLI Documentation](../docs/content/docs/cli-reference.mdx)
- [SDK Documentation](../docs/content/docs/sdk/)
- [App-Signal Rotation](../docs/content/docs/usage/app-signals.mdx)
- [Key Pools](../docs/content/docs/usage/key-pools.mdx)

