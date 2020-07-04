extern crate lambda_runtime;

use std::error::Error;
use scrappybotlib::bot;
use scrappybotlib::bot::BotStats;

use lambda_runtime::{error::HandlerError, lambda, Context};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
struct CustomEvent {
}

#[derive(Serialize, Deserialize)]
struct CustomOutput {
    changed_records: usize,
    new_records: usize,
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    lambda!(my_handler);

    Ok(())
}

pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use tokio::runtime;

    let mut rt = runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(future)
}


fn my_handler(e: CustomEvent, c: Context) -> Result<CustomOutput, HandlerError> {
    let bot_results:  Result<BotStats, Box<dyn std::error::Error>>  = block_on(bot::run());
    match bot_results {
        Ok(stats) => {
            Ok(CustomOutput {
                changed_records: stats.changed,
                new_records: stats.added,
                message: "Success".to_string(),
            })
        },
        Err(error) => {
            Err(HandlerError::from(&error.to_string()[..])) 
        }           
    }
}