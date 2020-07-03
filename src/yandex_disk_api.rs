use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub enum ResourceType {
    file,
    dir,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    pub _embedded: Option<ResourceList>,
    pub r#type: ResourceType,
    pub name: String,
    pub file: Option<String>,
    pub created: String,
    pub modified: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceList {
    pub sort: Option<String>,
    pub items: Vec<Resource>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub path: String,
    pub total: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResourceURLResponse {
    operation_id: Option<String>,
    href: String,
    method: String,
    templated: bool,
}

pub struct DiskClient {
    token: String,
    http_client: Client,
}

impl DiskClient {
    const API_URL: &'static str = "https://cloud-api.yandex.net/v1/disk";

    fn api_url(&self, method: &str) -> String {
        format!("{}/{}", DiskClient::API_URL, method)
    }

    pub fn new(token_value: String, http_client: Client) -> Self {
        DiskClient {
            token: token_value,
            http_client,
        }
    }

    pub async fn creat_new_file(&self, path: &str, data: Vec<u8>) -> Result<()> {
        let response_str = self
            .http_client
            .get(&self.api_url("resources/upload"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", self.token),
            )
            .query(&[("path", path)])
            .send()
            .await?
            .text()
            .await?;

        let response = serde_json::from_str::<ResourceURLResponse>(&response_str)
            .with_context(|| format!("Actual server response: `{}`", &response_str))?;

        let response_str = self
            .http_client
            .put(&response.href)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", self.token),
            )
            .body(data)
            .send()
            .await?
            .text()
            .await?;

        Ok(())
    }

    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let response_str = self
            .http_client
            .get(&self.api_url("resources/download"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", self.token),
            )
            .query(&[("path", path)])
            .send()
            .await?
            .text()
            .await?;

        let response = serde_json::from_str::<ResourceURLResponse>(&response_str)
            .with_context(|| format!("Actual response: `{}`", &response_str))?;

        Ok(self.read_url(&response.href).await?)
    }

    pub async fn read_file_from_resource(&self, resource: Resource) -> Result<Vec<u8>> {
        let url = &resource.file.context(format!(
            "Resource doesn't have a link to a file. (file = None)"
        ))?;
        Ok(self.read_url(url).await?)
    }

    pub async fn read_url(&self, url: &str) -> Result<Vec<u8>> {
        let response_str = self
            .http_client
            .get(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", self.token),
            )
            .send()
            .await?
            .bytes()
            .await?;

        Ok(response_str.to_vec())
    }

    pub async fn list_all_files(&self, folder: &str) -> Result<Vec<Resource>> {
        let response_str = self
            .http_client
            .get(&self.api_url("resources"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", self.token),
            )
            // .bearer_auth(&self.token)
            .query(&[
                ("limit", "1000"),
                ("path", folder),
                ("preview_crop", "true"),
            ])
            .send()
            .await?
            .text()
            .await?;

        let resource = serde_json::from_str::<Resource>(&response_str)
            .with_context(|| format!("Actual server response: `{}`", &response_str))?;

        Ok(resource._embedded.map_or(Vec::new(), |e| e.items))
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let response_str = self
            .http_client
            .delete(&self.api_url("resources"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("OAuth {}", self.token),
            )
            .query(&[("path", path)])
            .send()
            .await?
            .text()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const AUTH_KEY: &'static str = "TODO:<Yandex.API Key>";

    // const INTEGRATION_TEST_PREFIX: &'static str = "INTEGRATION_TEST";

    // use rand::{thread_rng, Rng};
    // use rand::distributions::Alphanumeric;

    // #[test]
    // fn test_yandex_disk_integration() {
    //     let client = reqwest::Client::new();
    //     let disk = DiskClient::new(AUTH_KEY.to_string(), client);
    //     let rand_string: String = thread_rng()
    //         .sample_iter(&Alphanumeric)
    //         .take(10)
    //         .collect();

    // }

    #[test]
    fn test_list_files() {
        let client = reqwest::Client::new();
        let disk = DiskClient::new(AUTH_KEY.to_string(), client);
        println!(
            "{:?}",
            tokio_test::block_on(disk.list_all_files("/estatebot"))
        );
    }

    #[test]
    fn test_create_file() {
        let client = reqwest::Client::new();
        let disk = DiskClient::new(AUTH_KEY.to_string(), client);
        println!(
            "{:?}",
            tokio_test::block_on(
                disk.creat_new_file("/estatebot/ololool2", "jojojopappa".as_bytes().to_vec())
            )
        );
    }

    #[test]
    fn test_read_file() {
        let client = reqwest::Client::new();
        let disk = DiskClient::new(AUTH_KEY.to_string(), client);
        println!(
            "{:?}",
            String::from_utf8(tokio_test::block_on(disk.read_file("/estatebot/first")).unwrap())
        );
    }
}
