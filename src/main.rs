extern crate anyhow;
extern crate glob;
extern crate google_drive3_fork as drive3;
extern crate select;
extern crate serde_json;
extern crate yup_oauth2 as oauth2;

mod api;
mod notification;
mod scrapes;
mod state;
mod storage;

use api::telegram_api::TelegramClient;
use state::Diff;
use state::Snapshot;
use storage::BlobStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = scrapes::hudhomestore::scrape().await?;

    let mut storage = storage::FsSystem::new();
    let old_snapshot = storage.load::<Snapshot>()?;

    let http_client = reqwest::Client::new();
    let telegram_client = TelegramClient::new("TODO: TELEGRAM API KEY".to_string(), http_client);
    let mut telegram = notification::TelegramService::new(telegram_client);
    let snapshot = Snapshot::new(items.to_vec());

    match old_snapshot {
        Some(old) => {
            let diff = old.diff(items);
            if !diff.changed.is_empty() || !diff.added.is_empty() {
                telegram.notify(diff, "hudhome listing").await?;

                storage.save(&snapshot)?;
            }
        }
        None => {
            telegram
                .notify(
                    Diff {
                        added: items,
                        changed: Vec::new(),
                    },
                    "hudhome listing",
                )
                .await?;

            storage.save(&snapshot)?;
        }
    }
    Ok(())
}
