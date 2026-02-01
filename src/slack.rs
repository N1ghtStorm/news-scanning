use serde::Serialize;

#[derive(Serialize)]
struct SlackPayload {
    text: String,
}

pub async fn post_news(webhook_url: &str, title: &str, url: &str) -> Result<(), String> {
    let text = format!("📰 *{}*\n<{}|Открыть>", title, url);

    let payload = SlackPayload { text };

    let res = reqwest::Client::new()
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Slack webhook error {}: {}", status, body));
    }

    Ok(())
}
