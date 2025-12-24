use anyhow::Context;
use serde::{Deserialize, Serialize};

pub struct HomeBox {
    base_url: String,
    client: reqwest::Client,
}

impl HomeBox {
    pub async fn new(
        base_url: &str,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();

        let auth = Self::user_login(
            &base_url,
            UserLoginRequest {
                username: username.to_string(),
                password: password.to_string(),
                stay_logged_in: false,
            },
        )
        .await
        .context("failed to authenticate homebox")?;

        let mut headers = reqwest::header::HeaderMap::new();
        let mut auth_value =
            reqwest::header::HeaderValue::from_str(&auth.token)?;
        auth_value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("failed to build homebox client")?;

        Ok(HomeBox { base_url, client })
    }

    async fn user_login(
        base_url: &str,
        request: UserLoginRequest,
    ) -> anyhow::Result<UserLoginResponse> {
        let url = format!("{}/api/v1/users/login", base_url);

        let client = reqwest::Client::builder()
            .build()
            .context("failed to instantiate user login client")?;

        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("failed to send user login request")?;

        let data = response
            .error_for_status()
            .context("invalid user login status code")?
            .json()
            .await
            .context("failed to deserialise user login response")?;

        Ok(data)
    }

    pub async fn get_item(
        &self,
        id: &str,
    ) -> anyhow::Result<HomeBoxItem> {
        let url = format!("{}/api/v1/items/{}", self.base_url, id);

        let response = self.client.get(&url).send().await.context(format!(
            "failed to send get item by id '{}' request",
            id
        ))?;

        let data: HomeBoxItem = response
            .error_for_status()
            .context(format!(
                "invalid get item by id '{}' status code",
                id
            ))?
            .json()
            .await
            .context(format!(
                "failed to deserialise get item by id '{}' response",
                id
            ))?;

        Ok(data)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeBoxItem {
    pub asset_id: String,
    pub id: String,
    pub name: String,
    pub quantity: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserLoginRequest {
    pub password: String,
    pub stay_logged_in: bool,
    pub username: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserLoginResponse {
    pub token: String,
}
