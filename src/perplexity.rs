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
    #[allow(dead_code)]
    pub snippet: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub date: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub last_updated: String,
}

const MAX_RESULTS_CAP: u32 = 100;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

pub async fn completions(api_key: &str, cfg: &PerplexityConfig) -> Result<String, String> {
    let body = ChatCompletionsRequest {
        model: "sonar".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: cfg.query.clone(),
        }],
        max_tokens: Some(1024),
    };

    let res = reqwest::Client::new()
        .post(format!("{}/chat/completions", API_BASE))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        let short = text.lines().next().unwrap_or(&text);
        let msg = if status.as_u16() == 401 {
            format!(
                "{} — check PERPLEXITY_API_KEY in .env",
                short
            )
        } else {
            format!("Perplexity API error {}: {}", status, short)
        };
        return Err(msg);
    }

    let out: ChatCompletionsResponse = res.json().await.map_err(|e| e.to_string())?;
    let text = out
        .choices
        .first()
        .and_then(|c| c.message.content.as_ref())
        .cloned()
        .unwrap_or_default();
    Ok(text)
}

pub async fn search(api_key: &str, cfg: &PerplexityConfig) -> Result<SearchResponse, String> {
    let max_results = cfg.max_results.min(MAX_RESULTS_CAP).max(1);
    let body = SearchRequest {
        query: cfg.query.clone(),
        max_results: Some(max_results),
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
        let short = text.lines().next().unwrap_or(&text);
        let msg = if status.as_u16() == 401 {
            format!(
                "{} — check PERPLEXITY_API_KEY in .env: pplx-... key from https://www.perplexity.ai/settings/api",
                short
            )
        } else {
            format!("Perplexity API error {}: {}", status, short)
        };
        return Err(msg);
    }

    let out: SearchResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(out)
}
