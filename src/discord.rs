use std::collections::HashMap;
use crate::Record;

// Discord API
#[tokio::main]
pub async fn send_discord_alert(webhook_url: String, newest: &Record, oldest: &Record, percent_diff: &String) -> Result<String, reqwest::Error> {
    let mut map = HashMap::new();
    map.insert("content", &percent_diff);

    let client = reqwest::Client::new();
    let res = client.post(&webhook_url)
                        .json(&map)
                        .send()
                        .await?;
    let body = res.text().await?;

    Ok(body)
}
