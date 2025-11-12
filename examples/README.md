# Keystone Examples

## App Signal Hook

This example demonstrates how to integrate Keystone with your application to trigger automatic secret rotation when rate limits are detected.

### Running the Example

1. Start the Keystone daemon:

```bash
keystone daemon start
```

2. Run the example:

```bash
cargo run --example app_signal_hook
```

### Integration in Your Application

#### Node.js / Express

```javascript
const axios = require('axios');

async function rotateSecretOnRateLimit(secretName) {
  try {
    const response = await axios.post('http://127.0.0.1:9123/rotate', {
      secret_name: secretName,
      env: process.env.NODE_ENV || 'prod',
      service: 'vercel'
    });
    
    console.log('Rotation queued:', response.data);
  } catch (error) {
    console.error('Failed to trigger rotation:', error.message);
  }
}

app.use((err, req, res, next) => {
  if (err.status === 429) {
    rotateSecretOnRateLimit('MY_API_KEY');
  }
  next(err);
});
```

#### Python / Flask

```python
import requests

def rotate_secret_on_rate_limit(secret_name: str):
    try:
        response = requests.post('http://127.0.0.1:9123/rotate', json={
            'secret_name': secret_name,
            'env': 'prod',
            'service': 'vercel'
        })
        print(f"Rotation queued: {response.json()}")
    except Exception as e:
        print(f"Failed to trigger rotation: {e}")

@app.errorhandler(429)
def handle_rate_limit(error):
    rotate_secret_on_rate_limit('MY_API_KEY')
    return "Rate limit exceeded", 429
```

#### Go

```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
)

type RotateRequest struct {
    SecretName string  `json:"secret_name"`
    Env        string  `json:"env"`
    Service    *string `json:"service,omitempty"`
}

func rotateSecretOnRateLimit(secretName string) error {
    req := RotateRequest{
        SecretName: secretName,
        Env:        "prod",
        Service:    stringPtr("vercel"),
    }
    
    body, _ := json.Marshal(req)
    _, err := http.Post(
        "http://127.0.0.1:9123/rotate",
        "application/json",
        bytes.NewBuffer(body),
    )
    
    return err
}

func middleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        next.ServeHTTP(w, r)
        
        if w.Header().Get("X-RateLimit-Remaining") == "0" {
            go rotateSecretOnRateLimit("MY_API_KEY")
        }
    })
}

func stringPtr(s string) *string {
    return &s
}
```

### Response Format

Success (202 Accepted):

```json
{
  "success": true,
  "message": "Rotation queued"
}
```

Rate limited (429 Too Many Requests):

```json
{
  "success": false,
  "message": "Cooldown active: 45s remaining"
}
```

Error (500 Internal Server Error):

```json
{
  "success": false,
  "message": "Failed to initialize audit logger: ..."
}
```

### Best Practices

1. **Debouncing**: Keystone automatically debounces signals using the configured cooldown period
2. **Async Handling**: Rotation happens asynchronously; don't block on the response
3. **Error Handling**: Log failures but don't crash the application
4. **Monitoring**: Monitor rotation audit logs to track automatic rotations
5. **Testing**: Test the signal endpoint in development before deploying

