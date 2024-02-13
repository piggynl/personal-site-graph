use std::env;
use std::sync::Arc;

use anyhow::Result;
use reqwest::Url;
use spdlog::prelude::*;

use crate::page::Page;

mod page;
mod site;
mod storage;

#[tokio::main]
async fn main() -> Result<()> {
    let default_logger: Arc<Logger> = spdlog::default_logger();
    default_logger.set_level_filter(LevelFilter::All);

    let argv1 = env::args().nth(1).unwrap();
    let url = Url::parse(&argv1)?;
    let page = Page::from_url(url).await?;
    _ = page;
    return Ok(());
}
