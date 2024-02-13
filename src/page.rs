use std::path::{Path, PathBuf};

use anyhow::Result;
use reqwest::{
    header::{self, HeaderName},
    ClientBuilder, Proxy, StatusCode, Url,
};
use serde::{Deserialize, Serialize};
use spdlog::prelude::*;

use crate::storage;

pub fn domain(url: &Url) -> String {
    psl::domain_str(url.host_str().unwrap()).unwrap().into()
}

pub fn encode_url(url: &Url) -> String {
    url.as_str()
        .chars()
        .map(|c| match c {
            c if c.is_alphanumeric() || !c.is_ascii() => c.into(),
            '-' | '_' | '.' | '~' => c.into(),
            _ => format!("%{:2x}", c as u32),
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    #[serde(with = "serde_url")]
    url: Url,
    domain: String,

    #[serde(with = "serde_status_code")]
    status: StatusCode,
    content_type: Option<String>,
    etag: Option<String>,
    last_modified: Option<String>,

    len: usize,
    #[serde(skip)]
    content: String,
}

mod serde_url {
    use reqwest::Url;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(url.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|url| Url::parse(&url).map_err(de::Error::custom))
    }
}

mod serde_status_code {
    use reqwest::StatusCode;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(status.as_u16())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        u16::deserialize(deserializer)
            .and_then(|code| StatusCode::from_u16(code).map_err(de::Error::custom))
    }
}

impl Page {
    fn from_cache(metadata_path: &Path, content_path: &Path) -> Result<Self> {
        let metadata_str = storage::read(metadata_path)?;
        let mut page: Page = serde_json::from_str(&metadata_str)?;
        page.content = storage::read(content_path)?;
        Ok(page)
    }

    fn write_cache(&self, metadata_path: &Path, content_path: &Path) -> Result<()> {
        storage::write(content_path, &self.content)?;
        let metadata_str = serde_json::to_string(&self)?;
        storage::write(metadata_path, &metadata_str)?;
        Ok(())
    }

    pub async fn from_url(url: Url) -> Result<Self> {
        let domain = domain(&url);
        let encoded_url = encode_url(&url);

        let metadata_path =
            PathBuf::from_iter(&["data", &domain, &format!("{}.metadata.json", &encoded_url)]);
        let content_path =
            PathBuf::from_iter(&["data", &domain, &format!("{}.content.html", &encoded_url)]);

        if let Ok(page) = Page::from_cache(&metadata_path, &content_path) {
            return Ok(page);
        }

        let client = ClientBuilder::new()
            .proxy(Proxy::all("socks5://localhost:20170")?)
            .build()
            .unwrap();

        let resp = client.get(url.clone()).send().await?;
        let status = resp.status();

        let get_header = |key: HeaderName| {
            resp.headers()
                .get(key)
                .and_then(|s| s.to_str().ok())
                .map(|s| s.to_string())
        };
        let content_type = get_header(header::CONTENT_TYPE);
        let etag = get_header(header::ETAG);
        let last_modified = get_header(header::LAST_MODIFIED);

        info!(
            "request: url={} status={} content_type={} etag={} last_modified={}",
            url,
            resp.status().as_str(),
            &content_type.as_ref().map_or("", |s| s.as_str()),
            &etag.as_ref().map_or("", |s| s.as_str()),
            &last_modified.as_ref().map_or("", |s| s.as_str()),
        );

        let content = resp.text().await?;

        let page = Page {
            url,
            domain,
            status,
            content_type,
            etag,
            last_modified,
            len: content.len(),
            content,
        };
        page.write_cache(&metadata_path, &content_path)?;
        Ok(page)
    }
}
