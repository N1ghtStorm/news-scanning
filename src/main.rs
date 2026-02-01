mod config;
mod perplexity;
mod state;

use config::Config;
use perplexity::search;
use state::State;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::var("CONFIG").unwrap_or_else(|_| "config.json".to_string());
    let config: Config = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;

    let api_key = std::env::var("PERPLEXITY_API_KEY").map_err(|_| "Set PERPLEXITY_API_KEY")?;

    let binding = config.state_file.clone();
    let state_path = Path::new(&binding);
    let mut state = State::load(state_path);

    let mut ticker = interval(Duration::from_secs(config.scan_interval_secs));
    let arc_config = Arc::new(config);

    ticker.tick().await;

    loop {
        match search(&api_key, &arc_config.perplexity).await {
            Ok(resp) => {
                for item in resp.results {
                    if state.is_new(&item.url) {
                        println!("[NEW] {} | {}", item.title, item.url);
                    }
                    state.mark_seen(&item.url);
                }
                if let Err(e) = state.save(state_path) {
                    eprintln!("Failed to save state: {}", e);
                }
            }
            Err(e) => eprintln!("Perplexity error: {}", e),
        }
        ticker.tick().await;
    }
}
