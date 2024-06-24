use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct RailwayError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RailwayResponse<T> {
    #[serde(default)]
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<RailwayError>,
}

pub struct Railway;

impl Railway {
    pub async fn query<T: serde::de::DeserializeOwned + Default + std::fmt::Debug>(
        json: serde_json::Value,
        token: &str,
    ) -> Result<T> {
        debug!("Executing query: {json:#?}");

        let response = reqwest::Client::new()
            .post("https://backboard.railway.app/graphql/v2")
            .header("Authorization", format!("Bearer {token}"))
            .json(&json)
            .fetch_mode_no_cors()
            .send()
            .await?;

        let status = response.status();
        if status != 200 {
            return Err(Error::RailwayStatusFailure(
                status.as_u16(),
                response.text().await?,
            ));
        }

        let response: RailwayResponse<serde_json::Value> = response.json().await?;
        debug!("Output: {response:#?}");

        if !response.errors.is_empty() {
            Err(Error::Railway(
                response.errors.into_iter().map(|e| e.message).collect(),
            ))
        } else if let Some(data) = response.data {
            Ok(T::deserialize(&data)?)
        } else {
            Err(Error::RailwayDataMissing("no data returned for: {query}"))
        }
    }
}
