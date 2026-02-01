use crate::config::PerplexityConfig;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.perplexity.ai";

#[derive(Debug, Serialize)]
struct SearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_recency_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_domain_filter: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub snippet: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub last_updated: String,
}

pub async fn search(api_key: &str, cfg: &PerplexityConfig) -> Result<SearchResponse, String> {
    let body = SearchRequest {
        query: cfg.query.clone(),
        max_results: Some(cfg.max_results),
        search_recency_filter: cfg.search_recency_filter.clone(),
        search_domain_filter: cfg.search_domain_filter.clone(),
    };

    let res = reqwest::Client::new()
        .post(format!("{}/search", API_BASE))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Perplexity API error {}: {}", status, text));
    }

    let out: SearchResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(out)
}
