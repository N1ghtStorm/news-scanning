mod config;
mod perplexity;
mod slack;
mod state;

use config::{Config, PerplexityApi};
use perplexity::{completions, search};
use slack::{post_news, send_test_message};
use state::State;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config_path = std::env::var("CONFIG").unwrap_or_else(|_| "config.json".to_string());
    let config: Config = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;

    let Ok(api_key) = std::env::var("PERPLEXITY_API_KEY") else {
        println!("Perplexity disabled (PERPLEXITY_API_KEY not set). Exiting.");
        return Ok(());
    };

    let slack_token = std::env::var("SLACK_BOT_TOKEN").ok();
    let slack_default_channel = config
        .slack_channel
        .clone()
        .or_else(|| std::env::var("SLACK_CHANNEL").ok())
        .map(|c| {
            if c.starts_with('#') || c.starts_with('C') {
                c
            } else {
                format!("#{}", c)
            }
        });

    let binding = config.state_file.clone();
    let state_path = Path::new(&binding);
    let mut state = State::load(state_path);

    let arc_config = Arc::new(config);
    let check_interval_secs = arc_config.scan_interval_secs.min(60);
    let mut ticker = interval(Duration::from_secs(check_interval_secs));
    let mut last_run: Vec<Option<Instant>> = vec![None; arc_config.sources.len()];

    ticker.tick().await;

    loop {
        let now = Instant::now();
        for (i, source) in arc_config.sources.iter().enumerate() {
            let due = match last_run[i] {
                None => true,
                Some(t) => now.saturating_duration_since(t).as_secs() >= source.time * 60,
            };
            if !due {
                continue;
            }
            let cfg = source.to_perplexity_config(
                arc_config.max_results,
                arc_config.search_recency_filter.as_ref(),
            );
            let channel = source
                .slack_channel
                .as_ref()
                .or(arc_config.slack_channel.as_ref())
                .or(slack_default_channel.as_ref())
                .map(|c| {
                    if c.starts_with('#') || c.starts_with('C') {
                        c.clone()
                    } else {
                        format!("#{}", c)
                    }
                });

            match source.api {
                PerplexityApi::Completions => {
                    match completions(&api_key, &cfg).await {
                        Ok(text) => {
                            last_run[i] = Some(Instant::now());
                            let result_text = format!("📋 Completions\n\n{}", text);
                            if let (Some(ref token), Some(ref ch)) = (slack_token.as_ref(), channel) {
                                if let Err(e) = send_test_message(token, ch, &result_text).await {
                                    eprintln!("Slack (result to bot): {}", e);
                                }
                            }
                        }
                        Err(e) => eprintln!("Perplexity completions error ({}): {}", source.sites.join(", "), e),
                    }
                }
                PerplexityApi::Search => {
                    match search(&api_key, &cfg).await {
                        Ok(resp) => {
                            last_run[i] = Some(Instant::now());
                            if resp.results.is_empty() {
                                eprintln!(
                                    "Perplexity returned 0 results for query \"{}\" (domains: {}). Try broader query or different search_recency_filter.",
                                    source.query,
                                    source.domains().join(", ")
                                );
                            }
                            let sites_label = source.sites.join(", ");
                            let lines: Vec<String> = resp
                                .results
                                .iter()
                                .enumerate()
                                .map(|(j, item)| format!("{}. <{}|{}>", j + 1, item.url, item.title))
                                .collect();
                            let result_text = if lines.is_empty() {
                                format!("📋 Query ({}): no news.", sites_label)
                            } else {
                                format!("📋 Query result ({})\n\n{}", sites_label, lines.join("\n"))
                            };
                            if let (Some(ref token), Some(ref ch)) = (slack_token.as_ref(), channel) {
                                if let Err(e) = send_test_message(token, ch, &result_text).await {
                                    eprintln!("Slack (result to bot): {}", e);
                                }
                            }
                            for item in resp.results {
                                if state.is_new(&item.url) {
                                    println!("[NEW] {} | {}", item.title, item.url);
                                    if let Some(ref webhook) = arc_config.slack_webhook_url {
                                        if !webhook.contains("YOUR/WEBHOOK") {
                                            if let Err(e) = post_news(webhook, &item.title, &item.url).await
                                            {
                                                eprintln!("Slack webhook error: {}", e);
                                            } else {
                                                println!("  -> sent to Slack");
                                            }
                                        }
                                    }
                                }
                                state.mark_seen(&item.url);
                            }
                        }
                        Err(e) => eprintln!("Perplexity error ({}): {}", source.sites.join(", "), e),
                    }
                }
            }
        }
        if let Err(e) = state.save(state_path) {
            eprintln!("Failed to save state: {}", e);
        }
        ticker.tick().await;
    }
}
