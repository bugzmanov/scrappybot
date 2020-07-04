use super::api::telegram_api::TelegramClient;
use super::api::yandex_disk_api::DiskClient;
use super::state::Diff;
use super::state::Snapshot;
use super::storage::BlobStorage;
use super::storage::YandexDiskStorage;
use super::scrapes;
use super::notification;

pub struct BotStats {
    pub changed: usize,
    pub added: usize,
}

impl BotStats {
    fn from_diff<T>(diff: &Diff<T>) -> Self {
        BotStats {
            changed: diff.changed.len(),
            added: diff.added.len()
        }
    }
}

pub async fn run() -> Result<BotStats, Box<dyn std::error::Error>> {
    let telegram_token = "TODO:".to_string();
    let yandex_token = "TODO:".to_string();
    let chat_id = "TODO:";

    let items = scrapes::hudhomestore::scrape().await?;

    // let mut storage = storage::FsSystem::new();
    let http_client = reqwest::Client::new();
    let disk_client = DiskClient::new(yandex_token, http_client);

    let storage = YandexDiskStorage::new(
        disk_client,
        "estatebot".to_string(),
        "hudhome_snapshot".to_string(),
    );
    let old_snapshot = storage.load::<Snapshot>().await?;

    let http_client = reqwest::Client::new();
    let telegram_client = TelegramClient::new(telegram_token, http_client);
    let mut telegram = notification::TelegramService::new(telegram_client, chat_id);
    let snapshot = Snapshot::new(items.to_vec());

    let stats = match old_snapshot {
        Some(old) => {
            let diff = old.diff(items);
            if !diff.changed.is_empty() || !diff.added.is_empty() {
                telegram.notify(&diff, "hudhome listing").await?;

                storage.save(&snapshot).await?
            }
            BotStats::from_diff(&diff)
        }
        None => {
            let diff = Diff {
                added: items,
                changed: Vec::new(),
            };

            telegram
                .notify(
                    &diff,
                    "hudhome listing",
                )
                .await?;

            storage.save(&snapshot).await?;
            BotStats::from_diff(&diff)
        }
    };
    Ok(stats)
}
