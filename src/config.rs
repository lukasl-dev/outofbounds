use std::{fs, path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MatrixMessageConfig {
    pub plain: String,
    pub html: String,
}

#[derive(Deserialize, Serialize)]
pub struct MatrixConfig {
    pub user: String,
    pub password: String,
    pub room_id: String,
    pub messages: Vec<MatrixMessageConfig>,
}

impl MatrixConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.user.is_empty() {
            anyhow::bail!("matrix user must not be empty");
        } else if self.password.is_empty() {
            anyhow::bail!("matrix password must not be empty");
        } else if self.room_id.is_empty() {
            anyhow::bail!("matrix room id must not be empty");
        } else if self.messages.is_empty() {
            anyhow::bail!("matrix messages must not be empty");
        } else {
            for message in &self.messages {
                if message.plain.is_empty() {
                    anyhow::bail!("matrix message plain must not be empty");
                } else if message.html.is_empty() {
                    anyhow::bail!("matrix message html must not be empty");
                }
            }
            Ok(())
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct HomeBoxItemConfig {
    pub asset_id: String,
    pub threshold: i32,
}

#[derive(Deserialize, Serialize)]
pub struct HomeBoxConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub items: Vec<HomeBoxItemConfig>,
}

impl HomeBoxConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.base_url.is_empty() {
            anyhow::bail!("homebox base url must not be empty");
        } else if self.username.is_empty() {
            anyhow::bail!("homebox username must not be empty");
        } else if self.password.is_empty() {
            anyhow::bail!("homebox password must not be empty");
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub matrix: MatrixConfig,
    pub homebox: HomeBoxConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            matrix: MatrixConfig {
                user: "@bot:example.com".to_string(),
                password: "".to_string(),
                room_id: "aslkdfasdlkfj1234a:example.com".to_string(),
                messages: vec![MatrixMessageConfig {
                    plain: "⚠️ Low stock: {name} (Quantity: {quantity}, Threshold: {threshold})".to_string(),
                    html: "⚠️ <b>Low stock</b>: <code>{name}</code> (Quantity: <b>{quantity}</b>, Threshold: <i>{threshold}</i>)".to_string(),
                }],
            },
            homebox: HomeBoxConfig {
                base_url: "https://demo.homebox.software".to_string(),
                username: "foo".to_string(),
                password: "baz".to_string(),
                items: vec![HomeBoxItemConfig {
                    asset_id: "000-001".to_string(),
                    threshold: 5,
                }],
            },
        }
    }
}

impl Config {
    pub fn load(path_str: &str) -> anyhow::Result<Self> {
        let path = path::Path::new(path_str);

        if path.exists() {
            let content = fs::read_to_string(path_str).context(format!(
                "failed to read config from '{}'",
                path_str
            ))?;

            let serialised: Self = toml::from_str(&content)
                .context(format!("failed to parse from '{}'", path_str))?;

            serialised.validate()?;
            Ok(serialised)
        } else {
            let defaults = Self::default();

            let as_toml = toml::to_string_pretty(&defaults)
                .context("failed to serialise default config")?;

            fs::write(path_str, as_toml).context(format!(
                "failed to write default config to '{}'",
                path_str
            ))?;

            Ok(defaults)
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        self.matrix.validate()?;
        self.homebox.validate()?;

        Ok(())
    }
}
