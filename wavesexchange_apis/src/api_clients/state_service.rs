use self::dto::*;
use crate::models::DataEntry;
use crate::{BaseApi, Error, HttpClient};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use wavesexchange_log::debug;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum HistoryPeg {
    Height(u32),
    Timestamp(String),
}

#[derive(Clone)]
pub struct StateSvcApi(Box<HttpClient<Self>>);

impl BaseApi for StateSvcApi {
    fn new_http(cli: &HttpClient<Self>) -> Self {
        StateSvcApi(Box::new(cli.clone()))
    }
}

impl StateSvcApi {
    pub async fn get_state(
        &self,
        address: impl AsRef<str>,
        key: impl AsRef<str>,
        history_peg: Option<HistoryPeg>,
    ) -> Result<Option<DataEntry>, Error> {
        let key_encoded = utf8_percent_encode(key.as_ref(), NON_ALPHANUMERIC);
        let url = match history_peg {
            None => {
                format!("entries/{}/{}", address.as_ref(), key_encoded,)
            }
            Some(HistoryPeg::Height(height)) => {
                format!(
                    "entries/{}/{}?height={}",
                    address.as_ref(),
                    key_encoded,
                    height,
                )
            }
            Some(HistoryPeg::Timestamp(timestamp)) => {
                format!(
                    "entries/{}/{}?block_timestamp={}",
                    address.as_ref(),
                    key_encoded,
                    timestamp,
                )
            }
        };

        let req_start_time = chrono::Utc::now();

        let res = self.0.get(&url).send().await.map_err(|err| {
            Error::HttpRequestError(
                std::sync::Arc::new(err),
                "Failed to get data entries from the state-service".to_string(),
            )
        })?;

        let req_end_time = chrono::Utc::now();
        debug!(
            "state-service get request took {:?}ms, URL: {}",
            (req_end_time - req_start_time).num_milliseconds(),
            url,
        );

        match res.status() {
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            reqwest::StatusCode::OK => Ok(Some(res.json().await.map_err(|err| {
                Error::HttpRequestError(
                    std::sync::Arc::new(err),
                    "Failed to parse json on fetching data entries from the state-service"
                        .to_string(),
                )
            })?)),
            s => {
                let body = res.text().await.unwrap_or_else(|_| "".to_owned());
                Err(Error::InvalidStatus(
                    s,
                    format!("State-service GET request failed. Body: {}", body),
                ))
            }
        }
    }

    pub async fn search(
        &self,
        query: impl Into<serde_json::Value> + Send,
    ) -> Result<Vec<DataEntry>, Error> {
        let req_start_time = chrono::Utc::now();

        let res: StateSearchResult = self
            .0
            .post("search")
            .json(&query.into())
            .send()
            .await
            .map_err(|err| {
                Error::HttpRequestError(
                    std::sync::Arc::new(err),
                    "Failed to get data entries from the state-service".to_string(),
                )
            })?
            .json()
            .await
            .map_err(|err| {
                Error::HttpRequestError(
                    std::sync::Arc::new(err),
                    "Failed to parse json on fetching data entries from the state-service"
                        .to_string(),
                )
            })?;

        let req_end_time = chrono::Utc::now();
        debug!(
            "state-service search request took {:?}ms",
            (req_end_time - req_start_time).num_milliseconds()
        );

        Ok(res.entries)
    }
}

pub mod dto {
    use crate::models::DataEntry;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub(super) struct StateSearchResult {
        pub entries: Vec<DataEntry>,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::blockchains::MAINNET;
    use serde_json::json;

    pub fn mainnet_client() -> HttpClient<StateSvcApi> {
        HttpClient::from_base_url(MAINNET::state_service_url)
    }

    #[tokio::test]
    async fn single_asset_price_request() {
        let query = json!({
            "filter": {
                "in": {
                    "properties": [
                        {
                            "address": {}
                        },
                        {
                            "key": {}
                        }
                    ],
                    "values": [
                        ["3P8qJyxUqizCWWtEn2zsLZVPzZAjdNGppB1", "%s%s__price__UAH"]
                    ]
                }
            }
        });

        let entries = mainnet_client().search(query).await.unwrap();

        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn defo_assets_list() {
        let query = json!({
            "filter": {
                "and": [
                  {
                    "address": {
                      "value": "3PQEjFmdcjd6wf1TrpkHSuDAk3zbfLSeikb"
                    }
                  },
                  {
                    "fragment": {
                      "position": 0,
                      "type": "string",
                      "operation": "eq",
                      "value": "defoAsset"
                    }
                  },
                  {
                    "fragment": {
                      "position": 2,
                      "type": "string",
                      "operation": "eq",
                      "value": "config"
                    }
                  }
                ]
            }
        });

        let entries = mainnet_client().search(query).await.unwrap();

        assert!(entries.len() >= 9);
    }
}
