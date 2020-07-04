extern crate anyhow;
extern crate glob;
extern crate select;
extern crate serde_json;

mod api;
mod notification;
mod scrapes;
mod state;
mod storage;
mod bot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    bot::run().await?;
    Ok(())
}
