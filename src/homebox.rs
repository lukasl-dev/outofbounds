use anyhow::Context;
use serde::{Deserialize, Serialize};

pub struct HomeBox {
    base_url: String,
    client: reqwest::Client,
    retries: u32,
}

impl HomeBox {
    pub async fn new(
        base_url: &str,
        username: &str,
        password: &str,
        retries: u32,
    ) -> anyhow::Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();

        let auth = Self::user_login(
            &base_url,
            UserLoginRequest {
                username: username.to_string(),
                password: password.to_string(),
                stay_logged_in: false,
            },
            retries,
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

        Ok(HomeBox {
            base_url,
            client,
            retries,
        })
    }

    async fn user_login(
        base_url: &str,
        request: UserLoginRequest,
        retries: u32,
    ) -> anyhow::Result<UserLoginResponse> {
        let url = format!("{}/api/v1/users/login", base_url);

        let client = reqwest::Client::builder()
            .build()
            .context("failed to instantiate user login client")?;

        for i in 0..retries {
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(i)))
                    .await;
            }

            let response = match client.post(&url).json(&request).send().await {
                Ok(r) => r,
                Err(_) if i < retries - 1 => continue,
                Err(e) => {
                    return Err(anyhow::anyhow!(e)
                        .context("failed to send user login request"));
                }
            };

            match response.error_for_status() {
                Ok(r) => match r.json().await {
                    Ok(data) => return Ok(data),
                    Err(_) if i < retries - 1 => continue,
                    Err(e) => {
                        return Err(anyhow::anyhow!(e).context(
                            "failed to deserialise user login response",
                        ));
                    }
                },
                Err(_) if i < retries - 1 => continue,
                Err(e) => {
                    return Err(anyhow::anyhow!(e)
                        .context("invalid user login status code"));
                }
            }
        }

        unreachable!()
    }

    pub async fn get_item(&self, id: &str) -> anyhow::Result<HomeBoxItem> {
        let url = format!("{}/api/v1/items/{}", self.base_url, id);

        for i in 0..self.retries {
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(i)))
                    .await;
            }

            let response = match self.client.get(&url).send().await {
                Ok(r) => r,
                Err(_) if i < self.retries - 1 => continue,
                Err(e) => {
                    return Err(anyhow::anyhow!(e).context(format!(
                        "failed to send get item by id '{}' request",
                        id
                    )));
                }
            };

            match response.error_for_status() {
                Ok(r) => match r.json().await {
                    Ok(data) => return Ok(data),
                    Err(_) if i < self.retries - 1 => continue,
                    Err(e) => return Err(anyhow::anyhow!(e).context(format!(
                        "failed to deserialise get item by id '{}' response",
                        id
                    ))),
                },
                Err(_) if i < self.retries - 1 => continue,
                Err(e) => {
                    return Err(anyhow::anyhow!(e).context(format!(
                        "invalid get item by id '{}' status code",
                        id
                    )));
                }
            }
        }

        unreachable!()
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
