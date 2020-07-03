use crate::state::IdChecksum;
use core::fmt::Display;
use std::hash::{Hash, Hasher};

use reqwest::header::USER_AGENT;
use select::document::Document;
use select::predicate::{Class, Name};
use std::collections::hash_map::DefaultHasher;

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

impl IdChecksum for ListingItem {
    fn id_checksum(&self) -> (String, u64) {
        (self.id.clone(), self.checksum)
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

pub async fn scrape() -> Result<Vec<ListingItem>, Box<dyn std::error::Error>> {
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
