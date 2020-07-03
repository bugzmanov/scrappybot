use crate::api::telegram_api::{SendMessage, TelegramClient};
use crate::Diff;
use anyhow::Result;
use core::fmt::Display;

pub trait NotificationService {
    fn notify<T: Display>(&mut self, diff: Diff<T>) -> Result<()>;
}

pub struct TelegramService {
    client: TelegramClient,
    chat_id: String,
}

impl TelegramService {
    pub fn new(client: TelegramClient) -> Self {
        TelegramService {
            client: client,
            chat_id: "-426414832".to_string(),
        }
    }

    pub async fn notify<T: Display>(&mut self, diff: Diff<T>, desc: &str) -> Result<()> {
        if !diff.added.is_empty() {
            for item in diff.added {
                let message = SendMessage {
                    chat_id: self.chat_id.clone(),
                    text: format!("New {}:\n {}", desc, item),
                    parse_mode: Some("MarkdownV2".to_string()),
                    disable_web_page_preview: true,
                };
                self.client.send_message(&message).await?;
            }
        }

        if !diff.changed.is_empty() {
            for item in diff.changed {
                let message = SendMessage {
                    chat_id: self.chat_id.clone(),
                    text: format!("Modified {}:\n {}", desc, item),
                    parse_mode: Some("MarkdownV2".to_string()),
                    disable_web_page_preview: true,
                };
                self.client.send_message(&message).await?;
            }
        }

        Ok(())
    }
}
