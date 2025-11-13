export interface BirchConfig {
  daemonUrl: string;
  environment: string;
  service?: string;
  enabled: boolean;
  debug?: boolean;
}

let globalConfig: BirchConfig | null = null;

export function detectService(): string | undefined {
  if (typeof process === 'undefined') return undefined;
  
  const env = process.env;
  
  if (env.VERCEL) return 'vercel';
  if (env.NETLIFY_SITE_ID) return 'netlify';
  if (env.RENDER_SERVICE_ID) return 'render';
  if (env.CF_PAGES) return 'cloudflare';
  if (env.FLY_APP_NAME) return 'fly';
  
  return undefined;
}

export async function checkDaemonHealth(daemonUrl: string): Promise<boolean> {
  try {
    const response = await fetch(`${daemonUrl}/health`, {
      signal: AbortSignal.timeout(2000)
    });
    return response.ok;
  } catch {
    return false;
  }
}

export async function autoDetectConfig(): Promise<BirchConfig> {
  const daemonUrl = 
    (typeof process !== 'undefined' && process.env.BIRCH_DAEMON_URL) || 
    'http://localhost:9123';
  
  const environment = 
    (typeof process !== 'undefined' && 
     (process.env.BIRCH_ENV || process.env.NODE_ENV)) || 
    'dev';
  
  const service = detectService();
  const enabled = await checkDaemonHealth(daemonUrl);
  
  const debug = 
    typeof process !== 'undefined' && 
    process.env.BIRCH_DEBUG === 'true';
  
  return {
    daemonUrl,
    environment,
    service,
    enabled,
    debug
  };
}

export function getConfig(): BirchConfig {
  if (!globalConfig) {
    throw new Error('@birch/client not initialized. Import "@birch/client/auto" first.');
  }
  return globalConfig;
}

export function getConfigSafe(): BirchConfig | null {
  return globalConfig;
}

export function setConfig(config: BirchConfig): void {
  globalConfig = config;
}

export interface ConfigureOptions {
  daemonUrl?: string;
  environment?: string;
  service?: string;
  debug?: boolean;
}

export async function configureBirch(options: ConfigureOptions = {}): Promise<void> {
  const auto = await autoDetectConfig();
  
  globalConfig = {
    daemonUrl: options.daemonUrl || auto.daemonUrl,
    environment: options.environment || auto.environment,
    service: options.service || auto.service,
    enabled: await checkDaemonHealth(options.daemonUrl || auto.daemonUrl),
    debug: options.debug || auto.debug
  };
}

