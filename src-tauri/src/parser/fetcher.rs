use reqwest::Client;
use std::time::Duration;

use crate::error::{AppError, AppResult};

pub struct FetchResult {
    pub html: String,
    pub url: String,
    pub status_code: u16,
}

pub async fn fetch_page(url: &str) -> AppResult<FetchResult> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;

    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::Parse(
            format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")),
        ));
    }

    let final_url = response.url().to_string();
    let html = response.text().await?;

    Ok(FetchResult {
        html,
        url: final_url,
        status_code: status.as_u16(),
    })
}
