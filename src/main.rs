extern crate anyhow;
extern crate glob;
extern crate google_drive3_fork as drive3;
extern crate select;
extern crate serde_json;
extern crate yup_oauth2 as oauth2;

mod notification;
mod storage;
mod telegram_api;
mod yandex_disk_api;

use reqwest::header::USER_AGENT;
use select::document::Document;
use select::predicate::{Class, Name};

use notification::NotificationService;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use storage::BlobStorage;
use telegram_api::TelegramClient;
use core::fmt::Display;

#[derive(Clone)]
pub struct ListingItem {
    pub id: String,
    pub csv: Vec<String>,
    pub checksum: u64,
}

impl ListingItem {
    fn from(csv: Vec<String>) -> Self {
        let mut s = DefaultHasher::new();
        csv.hash(&mut s);
        let hash_code = s.finish();

        ListingItem {
            id: csv[1].clone(),
            csv: csv.clone(),
            checksum: hash_code,
        }
    }
}

impl Display for ListingItem {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut result = String::new();
        let url = format!("[{}](https://www.hudhomestore.com/Listing/PropertyDetails.aspx?caseNumber={}&sLanguage=ENGLISH)", self.csv[2], self.id);
        let item_str = format!("{} - {:?}", url, &self.csv[3..])
            .replacen("-", "\\-", 100)
            .replacen(".", "\\.", 100)
            .replacen("{", "\\{", 100)
            .replacen("}", "\\}", 100);
        write!(fmt, "{}\n", item_str);
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    update_timestamp: u64,
    state: HashMap<String, u64>,
}

pub struct Diff<T> {
    pub added: Vec<T>,
    pub changed: Vec<T>,
}

impl Snapshot {
    fn new(items: Vec<ListingItem>) -> Self {
        let mut map = HashMap::new();
        for item in items {
            map.insert(item.csv[1].clone(), item.checksum);
        }
        Snapshot {
            update_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time should be after UNIX EPOCH")
                .as_secs(),
            state: map,
        }
    }

    fn diff(&self, items: Vec<ListingItem>) -> Diff<ListingItem> {
        let mut changed: Vec<ListingItem> = Vec::new();
        let mut added: Vec<ListingItem> = Vec::new();
        for item in items {
            match self.state.get(&item.id) {
                Some(checksum) if *checksum != item.checksum => changed.push(item),
                None => added.push(item),
                _ => { /* do nothing */ }
            }
        }

        Diff {
            changed: changed,
            added: added,
        }
    }
}

async fn scrape() -> Result<Vec<ListingItem>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let resp = client.get("https://www.hudhomestore.com/Listing/PropertySearchResult.aspx?pageId=1&zipCode=&city=&county=&sState=GA&fromPrice=0&toPrice=0&fCaseNumber=&bed=0&bath=0&street=&buyerType=0&specialProgram=&Status=0&indoorAmenities=&outdoorAmenities=&housingType=&stories=&parking=&propertyAge=&OrderbyName=SCASENUMBER&OrderbyValue=ASC&sPageSize=100&sLanguage=ENGLISH")
        .header(USER_AGENT, "curl/7.54.0")
        .send()
        .await?
        .text()
        .await?;

    let document = Document::from(&resp[..]);

    let mut items = Vec::new();

    for node in document.find(Class("FormTableRow")) {
        let columns = node
            .find(Name("td"))
            .take(9)
            .map(|n| {
                n.text()
                    .replacen("\t", "", 100)
                    .replacen("\n", " ", 5)
                    .trim()
                    .to_string()
            })
            .collect::<Vec<_>>();
        items.push(ListingItem::from(columns));
    }
    Ok(items)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = scrape().await?;

    let mut storage = storage::FsSystem::new();
    let old_snapshot = storage.load::<Snapshot>()?;

    let http_client = reqwest::Client::new();
    let telegram_client = TelegramClient::new(
        "TODO: TELEGRAM API KEY".to_string(),
        http_client,
    );
    let mut telegram = notification::TelegramService::new(telegram_client);
    let snapshot = Snapshot::new(items.to_vec());

    match old_snapshot {
        Some(old) => {
            let diff = old.diff(items);
            if !diff.changed.is_empty() || !diff.added.is_empty() {
                telegram.notify(diff).await?;

                storage.save(&snapshot)?;
            }
        }
        None => {
            telegram
                .notify(Diff {
                    added: items,
                    changed: Vec::new(),
                })
                .await?;

            storage.save(&snapshot)?;
        }
    }
    Ok(())
}
