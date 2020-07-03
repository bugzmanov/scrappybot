use crate::api::yandex_disk_api::DiskClient;
use anyhow::Context;
use anyhow::Result;
use glob::glob;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;

pub trait BlobStorage {
    fn save<'lifetime, T: Serialize>(&self, data: &T) -> Result<()>;
    fn load<'lifetime, T: DeserializeOwned>(&self) -> Result<Option<T>>;
}
const FILES_PREFIX: &'static str = "estate_snapshot";

pub struct NamingSchema {
    prefix: String,
}

impl NamingSchema {
    pub fn new(prefix: String) -> Self {
        NamingSchema { prefix: prefix }
    }

    fn parse_seq_num(&self, item_name: &str) -> u32 {
        item_name
            .split("_")
            .last()
            .and_then(|z| z.parse::<u32>().ok())
            .unwrap_or(0)
    }

    fn get_latest(&self, existing: &mut Vec<String>) -> Option<String> {
        let mut filtered: Vec<&mut String> = existing
            .into_iter()
            .filter(|s| s.starts_with(&self.prefix))
            .collect();
        filtered.sort_by(|a, b| self.parse_seq_num(a).cmp(&self.parse_seq_num(b)));
        filtered.last().map(|s| (*s).to_string())
    }

    fn next(&self, existing: &mut Vec<String>) -> String {
        match self.get_latest(existing) {
            Some(item) => format!("{}_{}", self.prefix, self.parse_seq_num(&item) + 1),
            None => format!("{}_0", self.prefix),
        }
    }
}

pub struct FsSystem {
    pub folder: String,
}

#[allow(dead_code)]
impl FsSystem {
    pub fn new() -> Self {
        FsSystem {
            folder: "./".to_string(),
        }
    }

    fn parse_number(&self, file_name: &str) -> u32 {
        file_name
            .split("_")
            .last()
            .and_then(|z| z.parse::<u32>().ok())
            .unwrap_or(0)
    }

    fn list_files(&self) -> Result<Vec<String>> {
        let mut result = Vec::new();
        for entry in glob(&format!("{}{}_*", &self.folder, FILES_PREFIX))
            .with_context(|| format!("failed to read files from {}", &self.folder))?
        {
            if let Ok(path) = entry {
                result.push(path.display().to_string());
            }
        }
        result.sort_by(|a, b| self.parse_number(a).cmp(&self.parse_number(b)));
        Ok(result)
    }

    fn open_latest(&self) -> Result<Option<File>> {
        let result = self.list_files()?.last().and_then(|f| File::open(f).ok());
        Ok(result)
    }

    fn create_new(&self) -> Result<File> {
        let created = match self.list_files()?.last() {
            Some(file) => File::create(format!(
                "{}{}_{}",
                self.folder,
                FILES_PREFIX,
                self.parse_number(file) + 1
            )),

            None => File::create(format!("{}{}_0", self.folder, FILES_PREFIX)),
        };
        created.with_context(|| format!("failed to create new file"))
    }
}

impl BlobStorage for FsSystem {
    fn save<'lifetime, T: Serialize>(&self, data: &T) -> Result<()> {
        let serialized = serde_json::to_string(data)?;

        let mut file = self.create_new()?;
        file.write_all(serialized.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    fn load<'lifetime, T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let file = self.open_latest()?;

        let data = match file {
            Some(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents)?;
                Some(serde_json::from_str(&contents)?)
            }
            None => None,
        };

        Ok(data)
    }
}

pub struct YandexDiskStorage {
    client: DiskClient,
    schema: NamingSchema,
    disk_folder: String,
}

impl YandexDiskStorage {
    pub fn new(client: DiskClient, disk_folder: String, files_prefix: String) -> Self {
        YandexDiskStorage {
            client: client,
            schema: NamingSchema::new(files_prefix),
            disk_folder: disk_folder,
        }
    }

    async fn list_files(&self) -> Result<Vec<String>> {
        let list = self
            .client
            .list_all_files(&self.disk_folder)
            .await?
            .into_iter()
            .map(|item| item.name)
            .collect();

        Ok(list)
    }

    pub async fn save<'lifetime, T: Serialize>(&self, data: &T) -> Result<()> {
        let mut existing = self.list_files().await?;

        let filename = self.schema.next(&mut existing);
        let serialized = serde_json::to_string(data)?;
        self.client
            .creat_new_file(
                &format!("{}/{}", self.disk_folder, filename),
                serialized.as_bytes().to_vec(),
            )
            .await?;

        Ok(())
    }

    pub async fn load<'lifetime, T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut existing = self.list_files().await?;

        let filename = self.schema.get_latest(&mut existing);

        match filename {
            Some(f) => {
                let data = self
                    .client
                    .read_file(&format!("{}/{}", self.disk_folder, f))
                    .await?;
                let state = serde_json::from_slice::<T>(data.as_slice())?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }
}
