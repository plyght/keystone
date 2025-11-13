use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
struct CreateWorkspaceRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceResponse {
    workspace: Workspace,
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    id: Uuid,
    name: String,
    plan_tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProviderConfig {
    id: Uuid,
    workspace_id: Uuid,
    provider: String,
    mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateProviderConfigRequest {
    provider: String,
    mode: String,
    config: serde_json::Value,
}

pub async fn login(api_url: Option<String>) -> Result<()> {
    let url = api_url.unwrap_or_else(|| "https://api.birch.sh".to_string());

    println!("Login to Birch SaaS");
    println!("API URL: {}", url);
    println!();
    println!("Please provide your API key:");

    let api_key = dialoguer::Input::<String>::new()
        .with_prompt("API Key")
        .interact_text()?;

    let mut config = Config::load()?;
    config.mode = "saas".to_string();
    config.saas_api_url = Some(url.clone());
    config.saas_api_key = Some(api_key);
    config.save()?;

    println!("✓ Successfully logged in to Birch SaaS");
    println!("  API URL: {}", url);

    Ok(())
}

pub async fn workspace_create(name: String) -> Result<()> {
    let config = Config::load()?;

    if config.mode != "saas" {
        anyhow::bail!("Not in SaaS mode. Run 'birch saas login' first.");
    }

    let api_url = config.saas_api_url.context("SaaS API URL not configured")?;
    let api_key = config.saas_api_key.context("SaaS API key not configured")?;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/workspaces", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&CreateWorkspaceRequest { name: name.clone() })
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to create workspace: {}", response.status());
    }

    let workspace_response: WorkspaceResponse = response.json().await?;

    println!("✓ Created workspace: {}", workspace_response.workspace.name);
    println!("  ID: {}", workspace_response.workspace.id);
    println!("  Plan: {}", workspace_response.workspace.plan_tier);
    println!();
    println!(
        "Run 'birch saas workspace select {}' to use this workspace",
        workspace_response.workspace.id
    );

    Ok(())
}

pub async fn workspace_list() -> Result<()> {
    let config = Config::load()?;

    if config.mode != "saas" {
        anyhow::bail!("Not in SaaS mode. Run 'birch saas login' first.");
    }

    let api_url = config.saas_api_url.context("SaaS API URL not configured")?;
    let api_key = config.saas_api_key.context("SaaS API key not configured")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/workspaces", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to list workspaces: {}", response.status());
    }

    let workspaces: Vec<Workspace> = response.json().await?;

    if workspaces.is_empty() {
        println!("No workspaces found.");
        println!("Create one with: birch saas workspace create <name>");
        return Ok(());
    }

    println!("Workspaces:");
    for workspace in workspaces {
        let selected = config
            .saas_workspace_id
            .as_ref()
            .map(|id| id == &workspace.id.to_string())
            .unwrap_or(false);

        let marker = if selected { "→" } else { " " };

        println!(
            "{} {} - {} ({})",
            marker, workspace.id, workspace.name, workspace.plan_tier
        );
    }

    Ok(())
}

pub async fn workspace_select(id: String) -> Result<()> {
    let mut config = Config::load()?;

    if config.mode != "saas" {
        anyhow::bail!("Not in SaaS mode. Run 'birch saas login' first.");
    }

    config.saas_workspace_id = Some(id.clone());
    config.save()?;

    println!("✓ Selected workspace: {}", id);

    Ok(())
}

pub async fn provider_set(provider: String, mode: String) -> Result<()> {
    let config = Config::load()?;

    if config.mode != "saas" {
        anyhow::bail!("Not in SaaS mode. Run 'birch saas login' first.");
    }

    let workspace_id = config
        .saas_workspace_id
        .context("No workspace selected. Run 'birch saas workspace select <id>' first.")?;

    let api_url = config.saas_api_url.context("SaaS API URL not configured")?;
    let api_key = config.saas_api_key.context("SaaS API key not configured")?;

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/api/v1/workspaces/{}/providers",
            api_url, workspace_id
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&CreateProviderConfigRequest {
            provider: provider.clone(),
            mode: mode.clone(),
            config: serde_json::json!({}),
        })
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to configure provider: {}", response.text().await?);
    }

    println!("✓ Configured provider '{}' with mode '{}'", provider, mode);

    Ok(())
}

pub async fn provider_list() -> Result<()> {
    let config = Config::load()?;

    if config.mode != "saas" {
        anyhow::bail!("Not in SaaS mode. Run 'birch saas login' first.");
    }

    let workspace_id = config
        .saas_workspace_id
        .context("No workspace selected. Run 'birch saas workspace select <id>' first.")?;

    let api_url = config.saas_api_url.context("SaaS API URL not configured")?;
    let api_key = config.saas_api_key.context("SaaS API key not configured")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/api/v1/workspaces/{}/providers",
            api_url, workspace_id
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to list providers: {}", response.status());
    }

    let providers: Vec<ProviderConfig> = response.json().await?;

    if providers.is_empty() {
        println!("No providers configured.");
        println!("Configure one with: birch saas provider set <provider> --mode <mode>");
        return Ok(());
    }

    println!("Configured providers:");
    for provider in providers {
        println!("  {} - {}", provider.provider, provider.mode);
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn resolve_credential(provider: &str, secret_name: &str) -> Result<Option<String>> {
    let config = Config::load()?;

    if config.mode != "saas" {
        return Ok(None);
    }

    let workspace_id = match config.saas_workspace_id {
        Some(id) => id,
        None => return Ok(None),
    };

    let api_url = match config.saas_api_url {
        Some(url) => url,
        None => return Ok(None),
    };

    let api_key = match config.saas_api_key {
        Some(key) => key,
        None => return Ok(None),
    };

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/api/v1/workspaces/{}/credentials/{}/{}",
            api_url, workspace_id, provider, secret_name
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    #[derive(Deserialize)]
    struct CredentialResponse {
        value: String,
    }

    let cred_response: CredentialResponse = response.json().await?;
    Ok(Some(cred_response.value))
}
