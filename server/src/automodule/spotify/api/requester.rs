use anyhow::anyhow;
use anyhow::Context;
use reqwest::Method;
use serde::de::DeserializeOwned;

const API_BASE_PATH: &str = "https://api.spotify.com/v1";

pub struct ApiRequester {
    client: reqwest::Client,
}

impl ApiRequester {
    pub fn new() -> Self {
        let client = reqwest::Client::new();

        ApiRequester { client }
    }

    pub async fn get<T>(&self, url: &str, access_token: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let full_url = format!("{}{}", API_BASE_PATH, url);
        let response = self
            .client
            .request(Method::GET, full_url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Could not GET from spotify API.")?;

        if !response.status().is_success() {
            let status_code = response.status();
            let text = response
                .text()
                .await
                .context("Could not convert Spotify API response to text to get error message.")?;
            return Err(anyhow!(
                "Spotify API request was not successful - {} - {}",
                status_code,
                text
            ));
        }

        let result = response
            .json::<T>()
            .await
            .context("Could not deserialize response from Spotify API")?;

        Ok(result)
    }
}
