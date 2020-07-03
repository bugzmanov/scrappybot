use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub message_id: i64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub document: Option<Document>,
    pub chat: Chat,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct Document {
    pub file_id: String,
    pub file_name: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct SendMessage {
    pub chat_id: String,
    pub text: String,
    pub parse_mode: Option<String>,
    pub disable_web_page_preview: bool,
}

pub struct TelegramClient {
    token: String,
    http_client: Client,
}

impl TelegramClient {
    const BASE_TELEGRAM_API_URL: &'static str = "https://api.telegram.org/bot";

    fn api_url(&self, method: &str) -> String {
        format!(
            "{}{}/{}",
            TelegramClient::BASE_TELEGRAM_API_URL,
            self.token,
            method
        )
    }

    pub fn new(token_value: String, http_client: Client) -> TelegramClient {
        TelegramClient {
            token: token_value,
            http_client,
        }
    }

    pub async fn send_message(&self, message: &SendMessage) -> Result<TelegramResponse<Message>> {
        let json_body = serde_json::to_string(message).with_context(|| {
            format!(
                "Failed to serialize body to json for sending message {:?}",
                message
            )
        });

        let response_str = self
            .http_client
            .post(&self.api_url("sendMessage"))
            .body(json_body?.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await?
            .text()
            .await?;

        let response = serde_json::from_str::<TelegramResponse<Message>>(&response_str)
            .with_context(|| format!("Actual server response '{}'", response_str))?;

        Ok(response)
    }
}
