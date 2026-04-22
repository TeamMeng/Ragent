//! Web search tool (stub — production: use Serper/Tavily/Bing API).

use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebSearchTool {
    api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebSearchInput {
    pub query: String,
    pub max_results: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebSearchOutput {
    pub results: Vec<WebSearchResult>,
    pub query: String,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self { api_key: None }
    }

    pub fn with_api_key(key: impl Into<String>) -> Self {
        Self { api_key: Some(key.into()) }
    }

    pub fn name(&self) -> &str { "web_search" }

    pub fn description(&self) -> &str {
        "Search the web for information. Returns titles, URLs, and snippets."
    }

    pub async fn call(&self, args: WebSearchInput) -> Result<WebSearchOutput, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref key) = self.api_key {
            self.search_serper(&args.query, args.max_results.unwrap_or(5), key).await
        } else {
            warn!("No API key configured for web_search tool, returning stub");
            Ok(WebSearchOutput {
                results: vec![WebSearchResult {
                    title: "Search not configured".into(),
                    url: "https://example.com".into(),
                    snippet: "Set WEB_SEARCH_API_KEY environment variable to enable web search.".into(),
                }],
                query: args.query,
            })
        }
    }

    async fn search_serper(&self, query: &str, num: usize, api_key: &str) -> Result<WebSearchOutput, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "q": query,
            "num": num,
        });
        let resp = client
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", api_key)
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let results = data["organic"].as_array()
            .map(|arr| arr.iter().take(num).map(|item| WebSearchResult {
                title: item["title"].as_str().unwrap_or("").into(),
                url: item["link"].as_str().unwrap_or("").into(),
                snippet: item["snippet"].as_str().unwrap_or("").into(),
            }).collect())
            .unwrap_or_default();

        Ok(WebSearchOutput { results, query: query.into() })
    }
}
