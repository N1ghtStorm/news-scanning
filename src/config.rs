use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub perplexity: PerplexityConfig,
    #[serde(default = "default_interval")]
    pub scan_interval_secs: u64,
    #[serde(default = "default_state_file")]
    pub state_file: String,
    pub slack_webhook_url: Option<String>,
}

fn default_interval() -> u64 {
    300
}

fn default_state_file() -> String {
    "news-state.json".to_string()
}

#[derive(Debug, Deserialize)]
pub struct PerplexityConfig {
    pub query: String,
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    pub search_recency_filter: Option<String>,
    pub search_domain_filter: Option<Vec<String>>,
}

fn default_max_results() -> u32 {
    10
}
