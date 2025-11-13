# @birch/client

Zero-configuration automatic API key rotation for Node.js applications. Automatically detects rate limits and rotates to the next key in your pool with no code changes required.

## Features

- **Zero Configuration**: Single import enables automatic rotation
- **Zero Intrusion**: Works with existing fetch, axios, and other HTTP clients
- **Automatic Detection**: Detects which API keys are being used
- **429 Handling**: Automatically rotates on rate limit responses
- **Immediate Retry**: Retries failed requests with new keys
- **Pool Support**: Manages multiple keys for each API
- **Works Everywhere**: Development, staging, and production environments
- **Debug Mode**: Optional verbose logging

## Installation

```bash
npm install @birch/client
```

## Quick Start

### 1. Set Up Birch CLI

```bash
birch daemon start

birch pool init TIKTOK_API_KEY --keys "key1,key2,key3"
```

### 2. Add One Import

```typescript
import '@birch/client/auto';
```

### 3. Use Your APIs Normally

```typescript
const response = await fetch('https://api.tiktok.com/v1/videos', {
  headers: {
    Authorization: `Bearer ${process.env.TIKTOK_API_KEY}`
  }
});
```

When you hit a 429 response, Birch automatically:
1. Detects the API key being used
2. Calls the Birch daemon to rotate
3. Gets the next key from the pool
4. Retries your request immediately

## Usage Examples

### Next.js App Router

```typescript
import '@birch/client/auto';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'My App',
};

export default function RootLayout({ children }) {
  return <html><body>{children}</body></html>;
}
```

```typescript
export async function GET() {
  const response = await fetch('https://api.tiktok.com/v1/videos', {
    headers: {
      Authorization: `Bearer ${process.env.TIKTOK_API_KEY}`
    }
  });
  
  const data = await response.json();
  return Response.json(data);
}
```

### Express API

```typescript
import '@birch/client/auto';
import express from 'express';

const app = express();

app.get('/tweets', async (req, res) => {
  const response = await fetch('https://api.twitter.com/2/tweets', {
    headers: {
      Authorization: `Bearer ${process.env.TWITTER_API_KEY}`
    }
  });
  
  const data = await response.json();
  res.json(data);
});

app.listen(3000);
```

### CLI Script

```typescript
import '@birch/client/auto';

async function main() {
  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${process.env.OPENAI_API_KEY}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      model: 'gpt-4',
      messages: [{ role: 'user', content: 'Hello!' }]
    })
  });
  
  const data = await response.json();
  console.log(data);
}

main();
```

### With Axios

```typescript
import '@birch/client/auto';
import axios from 'axios';

const response = await axios.get('https://api.tiktok.com/v1/videos', {
  headers: {
    Authorization: `Bearer ${process.env.TIKTOK_API_KEY}`
  }
});

console.log(response.data);
```

## Configuration

### Environment Variables

```bash
BIRCH_DAEMON_URL=http://localhost:9123
BIRCH_ENV=production
BIRCH_DEBUG=true
```

### Manual Configuration

```typescript
import { configureBirch } from '@birch/client';

await configureBirch({
  daemonUrl: 'http://localhost:9123',
  environment: 'production',
  service: 'vercel',
  debug: true
});
```

## How It Works

### Automatic Environment Detection

The SDK automatically detects your environment:

- **Service**: Vercel, Netlify, Render, Cloudflare, Fly.io
- **Environment**: `BIRCH_ENV` or `NODE_ENV` or `dev`
- **Daemon URL**: `BIRCH_DAEMON_URL` or `http://localhost:9123`

### Environment Variable Tracking

When you make an API call, Birch:

1. Intercepts the request
2. Reads the `Authorization` header
3. Matches the token against `process.env`
4. Stores the mapping: `api.tiktok.com` → `TIKTOK_API_KEY`

### Rate Limit Handling

When a request returns 429:

1. Looks up which env var was used
2. Calls `/rotate` on the Birch daemon
3. Daemon pulls next key from pool
4. Returns new key immediately
5. SDK retries with new key
6. Daemon updates cloud secrets async

## Manual Mode

For advanced use cases, import individual functions:

```typescript
import { 
  installFetchInterceptor,
  daemonClient,
  envTracker
} from '@birch/client';

await configureBirch({
  daemonUrl: 'http://localhost:9123',
  environment: 'production'
});

installFetchInterceptor();

const result = await daemonClient.rotate('TIKTOK_API_KEY');
console.log('New key:', result.new_value);
```

## Debug Mode

Enable verbose logging:

```bash
BIRCH_DEBUG=true
```

Output:

```
[Birch] Auto-rotation initialized { environment: 'dev', service: undefined }
[Birch] Fetch interceptor installed
[Birch] Detected env var: TIKTOK_API_KEY for token ***2345
[Birch] Rate limit hit (429) for TIKTOK_API_KEY, triggering rotation...
[Birch] Rotation successful, retrying with new key ***6789
```

## API Reference

### Types

```typescript
interface BirchConfig {
  daemonUrl: string;
  environment: string;
  service?: string;
  enabled: boolean;
  debug?: boolean;
}

interface RotateResult {
  success: boolean;
  new_value?: string;
  pool_status?: PoolStatus;
  message?: string;
}

interface PoolStatus {
  total_keys: number;
  available_keys: number;
  exhausted_keys: number;
  current_index: number;
}
```

### Functions

```typescript
configureBirch(options?: ConfigureOptions): Promise<void>

installFetchInterceptor(): void
uninstallFetchInterceptor(): void
installAxiosInterceptor(): void

daemonClient.rotate(secretName: string): Promise<RotateResult>
daemonClient.checkHealth(): Promise<boolean>

envTracker.trackRequest(url: string, headers: HeadersInit): void
envTracker.getSecretName(url: string): string | undefined
envTracker.clear(): void
```

## Environment Setup

### Development

```bash
cat > .env << EOF
TIKTOK_API_KEY=sk_dev_abc123
TWITTER_API_KEY=xoxb_dev_xyz789
BIRCH_DEBUG=true
EOF

birch daemon start
birch pool init TIKTOK_API_KEY --keys "key1,key2,key3"

npm run dev
```

### Production (Vercel)

```bash
vercel env add TIKTOK_API_KEY
vercel env add TWITTER_API_KEY

birch pool init TIKTOK_API_KEY --keys "prod_key1,prod_key2,prod_key3"

vercel deploy
```

## Troubleshooting

### "Daemon not available"

Ensure the Birch daemon is running:

```bash
birch daemon start
curl http://localhost:9123/health
```

### "Could not detect secret name"

Enable debug mode to see detection:

```bash
BIRCH_DEBUG=true node script.js
```

If the SDK does not recognize your token format:
1. Ensure your environment variable name includes API_KEY, TOKEN, or SECRET
2. Match the token value exactly in your environment

### Keys Not Rotating

Check your pool status:

```bash
birch pool status TIKTOK_API_KEY
```

Ensure you have multiple keys in the pool.

## Security

- Keys are never logged (only last 4 chars in debug mode)
- New keys only returned over localhost
- Env var tracking doesn't store values
- Graceful degradation if daemon unavailable

## License

MIT

## Contributing

See the main Birch repository for contribution guidelines.

