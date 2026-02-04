use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Array of sources: list of sites + query per entry.
    pub sources: Vec<SourceEntry>,
    #[serde(default = "default_interval")]
    pub scan_interval_secs: u64,
    #[serde(default = "default_state_file")]
    pub state_file: String,
    pub slack_webhook_url: Option<String>,
    /// Slack channel ID or name (e.g. #channel) where to send Perplexity results. Overrides SLACK_CHANNEL env if set.
    pub slack_channel: Option<String>,
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    pub search_recency_filter: Option<String>,
}

fn default_interval() -> u64 {
    300
}

fn default_state_file() -> String {
    "news-state.json".to_string()
}

fn default_max_results() -> u32 {
    50
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourceEntry {
    /// List of domains to search (e.g. ["example.com", "other.com"]).
    pub sites: Vec<String>,
    /// Query to Perplexity for these sites.
    pub query: String,
    /// How many minutes until this query is run again (interval in minutes).
    #[serde(default = "default_time_minutes")]
    pub time: u64,
    /// Slack channel for this source's results (e.g. #crypto-news). Overrides global slack_channel if set.
    pub slack_channel: Option<String>,
}

fn default_time_minutes() -> u64 {
    60
}

fn url_to_domain(site: &str) -> String {
    let s = site.trim();
    let rest = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
        .unwrap_or(s);
    let domain = rest
        .split('/')
        .next()
        .and_then(|h| h.split('?').next())
        .unwrap_or(rest);
    if domain.is_empty() {
        s.to_string()
    } else {
        domain.to_string()
    }
}

impl SourceEntry {
    /// Domains only (no paths) for Perplexity search_domain_filter.
    pub fn domains(&self) -> Vec<String> {
        self.sites.iter().map(|s| url_to_domain(s)).collect()
    }

    pub fn to_perplexity_config(
        &self,
        max_results: u32,
        search_recency_filter: Option<&String>,
    ) -> PerplexityConfig {
        PerplexityConfig {
            query: self.query.clone(),
            max_results,
            search_recency_filter: search_recency_filter.cloned(),
            search_domain_filter: if self.sites.is_empty() {
                None
            } else {
                Some(self.domains())
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerplexityConfig {
    pub query: String,
    pub max_results: u32,
    pub search_recency_filter: Option<String>,
    pub search_domain_filter: Option<Vec<String>>,
}
