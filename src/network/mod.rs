use anyhow::{anyhow, Result};
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH};

#[derive(Debug, Clone)]
pub struct RemoteMetadata {
    pub content_length: Option<u64>,
    pub range_supported: bool,
}

#[derive(Clone)]
pub struct HttpClient {
    pub client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn probe(&self, url: &str) -> Result<RemoteMetadata> {
        let resp = self.client.head(url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("HEAD request failed: {}", resp.status()));
        }

        let content_length = resp
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        let range_supported = resp
            .headers()
            .get(ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("bytes"))
            .unwrap_or(false);

        Ok(RemoteMetadata {
            content_length,
            range_supported,
        })
    }
}
