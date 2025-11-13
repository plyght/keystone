import { getConfigSafe } from './config';

export class EnvTracker {
  private usedEnvVars = new Map<string, string>();
  private requestToSecret = new Map<string, string>();

  trackRequest(url: string, headers: HeadersInit | undefined): void {
    const secretName = this.detectSecretFromHeaders(headers);
    if (secretName) {
      const urlPattern = this.getUrlPattern(url);
      this.usedEnvVars.set(urlPattern, secretName);
      this.requestToSecret.set(url, secretName);
    }
  }

  getSecretName(url: string): string | undefined {
    const direct = this.requestToSecret.get(url);
    if (direct) return direct;

    const urlPattern = this.getUrlPattern(url);
    return this.usedEnvVars.get(urlPattern);
  }

  private getUrlPattern(url: string): string {
    try {
      const urlObj = new URL(url);
      return urlObj.hostname;
    } catch {
      return url;
    }
  }

  private detectSecretFromHeaders(headers: HeadersInit | undefined): string | undefined {
    if (!headers) return undefined;
    if (typeof process === 'undefined') return undefined;

    const authHeader = this.getAuthHeader(headers);
    if (!authHeader) return undefined;

    const token = authHeader.replace(/^Bearer\s+/i, '').trim();
    
    for (const [key, value] of Object.entries(process.env)) {
      if (value === token && (key.includes('API_KEY') || key.includes('TOKEN') || key.includes('SECRET'))) {
        const config = getConfigSafe();
        if (config?.debug) {
          console.log(`[Birch] Detected env var: ${key} for token ***${token.slice(-4)}`);
        }
        return key;
      }
    }

    const fallback = this.detectFromTokenPrefix(token);
    const config = getConfigSafe();
    if (fallback && config?.debug) {
      console.log(`[Birch] Fallback detection: ${fallback} for token prefix`);
    }
    return fallback;
  }

  private getAuthHeader(headers: HeadersInit): string | undefined {
    if (headers instanceof Headers) {
      return headers.get('Authorization') || headers.get('authorization') || undefined;
    }
    
    if (Array.isArray(headers)) {
      const authEntry = headers.find(([key]) => 
        key.toLowerCase() === 'authorization'
      );
      return authEntry ? authEntry[1] : undefined;
    }
    
    if (typeof headers === 'object') {
      return (headers as Record<string, string>)['Authorization'] || 
             (headers as Record<string, string>)['authorization'];
    }
    
    return undefined;
  }

  private detectFromTokenPrefix(token: string): string | undefined {
    if (token.startsWith('sk-') || token.startsWith('sk_')) {
      if (token.toLowerCase().includes('tiktok')) return 'TIKTOK_API_KEY';
      if (token.toLowerCase().includes('twitter')) return 'TWITTER_API_KEY';
      if (token.toLowerCase().includes('openai')) return 'OPENAI_API_KEY';
    }
    
    if (token.startsWith('xoxb-')) return 'SLACK_BOT_TOKEN';
    if (token.startsWith('ghp_')) return 'GITHUB_TOKEN';
    
    return undefined;
  }

  clear(): void {
    this.usedEnvVars.clear();
    this.requestToSecret.clear();
  }
}

export const envTracker = new EnvTracker();

