use reqwest;
use serde_json::json;
use std::time::Duration;
use tracing::{error, info};
use crate::error::*;

/// Load a model and keep it alive indefinitely
pub async fn load_model_indefinitely(url: &str, model_name: &str) -> ReaderResult<()> {
    let client = reqwest::Client::new();
    let api = format!("{}/api/chat", url);
    
    // Send a minimal generate request with keep_alive = -1
    let payload = json!({
        "model": model_name,
        "prompt": "",  // Empty prompt is fine - just loads the model
        "keep_alive": -1,
        "stream": false
    });

    let response = client
        .post(&api)
        .json(&payload)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    if response.status().is_success() {
        info!("✅ Model '{}' loaded and will stay alive indefinitely", model_name);
        Ok(())
    } else {
        let error_text = response.text().await?;
        error!("❌ Failed to load model: {}", error_text);
        Err(ReaderError::Ai(format!("Server returned error: {}", error_text)))
    }
}

/// Unload a model from memory immediately
pub async fn unload_model(url: &str, model_name: &str) -> ReaderResult<()> {
    let client = reqwest::Client::new();
    let api = format!("{}/api/chat", url);
    
    // Send request with keep_alive = 0 to unload immediately
    let payload = json!({
        "model": model_name,
        "prompt": "",  // Empty prompt - just triggers unload
        "keep_alive": 0,
        "stream": false
    });

    let response = client
        .post(&api)
        .json(&payload)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    if response.status().is_success() {
        info!("✅ Model '{}' unloaded from memory", model_name);
        Ok(())
    } else {
        let error_text = response.text().await?;
        error!("❌ Failed to unload model: {}", error_text);
        Err(ReaderError::Ai(format!("Server returned error: {}", error_text)))
    }
}
