use chrono::{DateTime, NaiveDateTime, Utc};
use livepeer_rs::vod::Vod;

pub fn generate_playback_policy(client: &livepeer_rs::Livepeer) -> Option<serde_json::Value> {
    let mut playback_policy: Option<serde_json::Value> = None;
    let mut select_playback_policy =
        dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .items(&["Public", "Private"])
            .default(0)
            .interact()
            .unwrap();
    if select_playback_policy == 1 {
        let mut select_playback_policy =
            dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .items(&["Webhook"])
                .default(0)
                .interact()
                .unwrap();
        if select_playback_policy == 0 {
            let webhooks = client.asset.list_webhooks().unwrap();
            let list = webhooks.as_array().unwrap();

            let ids = list
                .iter()
                .map(|x| {
                    let created_at = x["createdAt"].as_str().unwrap_or("");

                    let created_at_formatted = if !created_at.is_empty() {
                        let timestamp = created_at.parse::<i64>().unwrap();
                        let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);
                        let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
                        datetime.to_rfc3339()
                    } else {
                        String::from("")
                    };

                    format!(
                        "{} - {} - {} - {}",
                        x["name"].as_str().unwrap_or(""),
                        x["events"].as_str().unwrap_or(""),
                        x["url"].as_str().unwrap_or(""),
                        created_at_formatted
                    )
                })
                .collect::<Vec<String>>();

            let mut select_webhook =
                dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .items(&ids)
                    .default(0)
                    .interact()
                    .unwrap();

            playback_policy = Some(serde_json::json!({
                "type": "webhook",
                "webhookId": list[select_webhook]["id"].as_str().unwrap_or(""),
                "webhook_context": {

                }
            }));
        }
    }

    return playback_policy;
}
