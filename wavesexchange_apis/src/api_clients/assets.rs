use crate::{ApiResult, BaseApi, HttpClient};
use itertools::join;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

#[derive(Clone, Debug)]
pub struct AssetsService;

impl BaseApi for AssetsService {}

impl HttpClient<AssetsService> {
    pub async fn get(
        &self,
        asset_ids: impl IntoIterator<Item = impl Into<String>>,
        height: Option<u32>,
    ) -> ApiResult<dto::AssetResponse> {
        let url = match build_url(&self.base_url(), asset_ids, height) {
            Some(u) => u,
            None => return Ok(dto::AssetResponse { data: vec![] }),
        };

        self.create_req_handler(self.get_client().get(&url), "assets::get_assets")
            .execute()
            .await
    }
}

pub mod dto {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct AssetResponse {
        pub data: Vec<AssetData>,
    }

    #[derive(Debug, Deserialize)]
    pub struct AssetData {
        pub data: Asset,
    }

    #[derive(Debug, Deserialize)]
    pub struct Asset {
        pub id: String,
        pub quantity: i64,
    }
}

fn build_url(
    root_url: &str,
    asset_ids: impl IntoIterator<Item = impl Into<String>>,
    height: Option<u32>,
) -> Option<String> {
    let asset_ids = asset_ids
        .into_iter()
        .map(|id| utf8_percent_encode(&id.into(), NON_ALPHANUMERIC).to_string());
    let ids = join(asset_ids, "&ids=");
    if ids.is_empty() {
        return None;
    }
    let mut url = format!("{}?ids={}", root_url, ids);
    if let Some(height) = height {
        url.push_str("&height__gte=");
        url.push_str(&height.to_string());
    }
    Some(url)
}

// public exports for tests
pub mod tests {
    use super::*;
    use crate::tests::blockchains::MAINNET;

    pub fn mainnet_client() -> HttpClient<AssetsService> {
        HttpClient::from_base_url(MAINNET::assets_service_url)
    }
}

#[cfg(test)]
mod tests_internal {
    use super::tests::*;
    use super::*;

    #[test]
    fn test_build_url() {
        assert_eq!(build_url("http://assets", Vec::<String>::new(), None), None);
        assert_eq!(
            build_url("http://assets", Vec::<String>::new(), Some(1)),
            None
        );
        assert_eq!(
            build_url("http://assets", vec!["123"], None).unwrap(),
            "http://assets?ids=123"
        );
        assert_eq!(
            build_url("http://assets", vec!["123", "456"], None).unwrap(),
            "http://assets?ids=123&ids=456"
        );
        assert_eq!(
            build_url("http://assets", vec!["123", "456"], Some(789)).unwrap(),
            "http://assets?ids=123&ids=456&height__gte=789"
        );
        assert_eq!(
            build_url("http://assets", vec!["foo%"], None).unwrap(),
            "http://assets?ids=foo%25"
        );
    }

    #[tokio::test]
    async fn test_assets_get() {
        let resp = mainnet_client().get(vec!["WAVES"], Some(1)).await.unwrap();
        let data = &resp.data[0].data;
        assert_eq!(&data.id, "WAVES");
        assert_eq!(data.quantity, 10000000000000000);
    }
}
