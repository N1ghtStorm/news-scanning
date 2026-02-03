mod config;
mod perplexity;
mod slack;
mod state;

use config::Config;
use perplexity::search;
use slack::{post_news, send_test_message};
use state::State;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config_path = std::env::var("CONFIG").unwrap_or_else(|_| "config.json".to_string());
    let config: Config = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;

    let slack_channel = std::env::var("SLACK_CHANNEL").unwrap_or_else(|_| "#test-n".to_string());
    let channel = if slack_channel.starts_with('#') || slack_channel.starts_with('C') {
        slack_channel.clone()
    } else {
        format!("#{}", slack_channel)
    };
    if let Ok(token) = std::env::var("SLACK_BOT_TOKEN") {
        match send_test_message(&token, &channel, "Тестовое сообщение от news-scanning").await
        {
            Err(e) => eprintln!("Slack test message error: {}", e),
            Ok(()) => println!("Test message sent to Slack (channel: {})", channel.as_str()),
        }
    } else {
        println!("SLACK_BOT_TOKEN не задан в .env — тест в Slack пропущен");
    }

    let Ok(api_key) = std::env::var("PERPLEXITY_API_KEY") else {
        println!("Perplexity отключён (PERPLEXITY_API_KEY не задан). Работа завершена.");
        return Ok(());
    };

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
                        if let Some(ref webhook) = arc_config.slack_webhook_url {
                            if let Err(e) = post_news(webhook, &item.title, &item.url).await {
                                eprintln!("Slack error: {}", e);
                            } else {
                                println!("  -> отправлено в Slack");
                            }
                        }
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
