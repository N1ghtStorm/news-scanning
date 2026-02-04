use serde::Serialize;

const SLACK_API: &str = "https://slack.com/api/chat.postMessage";

#[derive(Serialize)]
struct ChatPostMessageBody<'a> {
    channel: &'a str,
    text: &'a str,
}

/// Send a message to a channel via Bot Token.
pub async fn send_test_message(token: &str, channel: &str, text: &str) -> Result<(), String> {
    let body = ChatPostMessageBody { channel, text };
    let res = reqwest::Client::new()
        .post(SLACK_API)
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Slack API error {}: {}", status, body));
    }

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    if !json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let err = json.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
        return Err(format!("Slack API error: {}", err));
    }

    Ok(())
}

#[derive(Serialize)]
struct WebhookPayload {
    text: String,
}

pub async fn post_news(webhook_url: &str, title: &str, url: &str) -> Result<(), String> {
    let text = format!("📰 *{}*\n<{}|Open>", title, url);

    let payload = WebhookPayload { text };

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
