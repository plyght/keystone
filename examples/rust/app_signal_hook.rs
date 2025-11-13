use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Birch App-Signal Hook Example\n");

    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

    println!("Simulating API rate limit detection...");

    let rate_limit_detected = true;

    if rate_limit_detected {
        println!("Rate limit detected! Triggering secret rotation via Birch daemon...\n");

        let response = client
            .post("http://127.0.0.1:9123/rotate")
            .json(&json!({
                "secret_name": "MY_API_KEY",
                "env": "prod",
                "service": "vercel"
            }))
            .send()
            .await?;

        let status = response.status();
        let body: serde_json::Value = response.json().await?;

        println!("Response status: {}", status);
        println!("Response body: {}", serde_json::to_string_pretty(&body)?);

        if status.is_success() {
            println!("\n✅ Rotation signal sent successfully!");
            println!("Birch daemon will process the rotation asynchronously.");
        } else {
            println!("\n❌ Failed to send rotation signal.");
        }
    }

    Ok(())
}

