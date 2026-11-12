use reqwest::{ClientBuilder, Method, Request, Response, Url};
use std::{collections::HashMap, ops::Deref};

use crate::store::Store;
use std::{num::TryFromIntError, path::PathBuf};

pub struct AppState {
    pub store: Store,
    pub client: Option<HttpClient>,
}

#[derive(Clone)]
pub struct HttpClient {
    url: Url,
    client: reqwest::Client,
}

impl Deref for HttpClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl HttpClient {
    pub fn new(url: Url) -> Result<Self, HttpError> {
        let client = ClientBuilder::new().build()?;
        Ok(Self { url, client })
    }

    pub fn path(&self, path: impl AsRef<str>) -> Result<Url, HttpError> {
        let url = self.url.join(path.as_ref())?;
        Ok(url)
    }

    pub fn construct<K, V>(
        &self,
        path: impl AsRef<str>,
        params: Option<HashMap<K, V>>,
    ) -> Result<Request, HttpError>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut url = self.url.join(path.as_ref())?;
        if let Some(params) = params {
            url.query_pairs_mut()
                .clear()
                .extend_pairs(params.iter())
                .finish();
        }
        Ok(self.client.request(Method::GET, url).build()?)
    }

    pub async fn send(&self, req: Request) -> Result<Response, HttpError> {
        Ok(self.client.execute(req).await?)
    }
}

pub struct LogFiles {
    pub healthmeter: Vec<PathBuf>,
    pub cpumonitoring: Vec<PathBuf>,
    pub cpumemstats: Vec<PathBuf>,
    pub stuckthreads: Vec<PathBuf>,
    pub stuckqueries: Vec<PathBuf>,
    pub runningqueries: Vec<PathBuf>,
    pub connectiondump: Vec<PathBuf>,
    pub threaddump: Vec<PathBuf>,
    pub fragmentation_ratio: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("Invalid URL")]
    URLParse(#[from] url::ParseError),

    #[error("Cannot make/construct HTTP request: {0}")]
    RequestError(#[from] reqwest::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum TimestampError {
    #[error("Time/Date Parsing failed: {0}")]
    Parse(#[from] time::error::Parse),
    #[error("Conversion to u64 failed: {0}")]
    Conversion(#[from] TryFromIntError),
    #[error("Normalizing Timestamp with Timezone Info failed")]
    InvalidTime,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseInt {
    #[error("Invalid Digit found during parsing: {0}")]
    InvalidDigit(u8),
}
