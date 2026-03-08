use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::json;

use crate::Result;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct DiscordClient {
    client: reqwest::Client,
    api_base: String,
}

impl DiscordClient {
    pub fn from_config(config: Arc<AppConfig>) -> Result<Self> {
        let token = config
            .effective_token()
            .ok_or_else(|| "missing Discord bot token; configure ~/.clawhip/config.toml or DISCORD_TOKEN (CLAWHIP_DISCORD_BOT_TOKEN is also supported)".to_string())?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bot {token}"))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let api_base = std::env::var("CLAWHIP_DISCORD_API_BASE")
            .unwrap_or_else(|_| "https://discord.com/api/v10".to_string());

        Ok(Self { client, api_base })
    }

    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<()> {
        let url = format!(
            "{}/channels/{}/messages",
            self.api_base.trim_end_matches('/'),
            channel_id
        );
        let response = self
            .client
            .post(url)
            .json(&json!({ "content": content }))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("Discord API request failed with {status}: {body}").into())
    }
}
