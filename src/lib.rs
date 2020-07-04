#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate glob;
#[macro_use]
extern crate select;
#[macro_use]
extern crate serde_json;

mod api;
mod notification;
mod scrapes;
mod state;
mod storage;
pub mod bot;