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
    pub password: Option<String>,
    pub password_file: Option<String>,
    pub room_id: String,
    #[serde(default = "default_retries")]
    pub retries: u32,
    pub messages: Vec<MatrixMessageConfig>,
}

fn default_retries() -> u32 {
    5
}

impl MatrixConfig {
    pub fn resolve_password(&self) -> anyhow::Result<String> {
        if let Some(file) = &self.password_file {
            fs::read_to_string(file)
                .map(|s| s.trim().to_string())
                .context(format!(
                    "failed to read matrix password from '{}'",
                    file
                ))
        } else if let Some(password) = &self.password {
            Ok(password.clone())
        } else {
            anyhow::bail!(
                "either matrix password or password_file must be set"
            );
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.user.is_empty() {
            anyhow::bail!("matrix user must not be empty");
        } else if self.password.as_deref().unwrap_or("").is_empty()
            && self.password_file.as_deref().unwrap_or("").is_empty()
        {
            anyhow::bail!(
                "either matrix password or password_file must be set"
            );
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
    pub id: String,
    pub threshold: i32,
}

#[derive(Deserialize, Serialize)]
pub struct HomeBoxConfig {
    pub base_url: String,
    pub username: String,
    pub password: Option<String>,
    pub password_file: Option<String>,
    #[serde(default = "default_retries")]
    pub retries: u32,
    pub items: Vec<HomeBoxItemConfig>,
}

impl HomeBoxConfig {
    pub fn resolve_password(&self) -> anyhow::Result<String> {
        if let Some(file) = &self.password_file {
            fs::read_to_string(file)
                .map(|s| s.trim().to_string())
                .context(format!(
                    "failed to read homebox password from '{}'",
                    file
                ))
        } else if let Some(password) = &self.password {
            Ok(password.clone())
        } else {
            anyhow::bail!(
                "either homebox password or password_file must be set"
            );
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.base_url.is_empty() {
            anyhow::bail!("homebox base url must not be empty");
        } else if self.username.is_empty() {
            anyhow::bail!("homebox username must not be empty");
        } else if self.password.as_deref().unwrap_or("").is_empty()
            && self.password_file.as_deref().unwrap_or("").is_empty()
        {
            anyhow::bail!(
                "either homebox password or password_file must be set"
            );
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
                password: Some("".to_string()),
                password_file: None,
                room_id: "aslkdfasdlkfj1234a:example.com".to_string(),
                retries: default_retries(),
                messages: vec![
                    MatrixMessageConfig {
                        plain: "⚠️ Alarm! Von {name} haben wir nur noch {quantity} da (Limit: {threshold}). Ab zum Einkaufen!".to_string(),
                        html: "⚠️ <b>Großer Notfall!</b> Von <code>{name}</code> haben wir nur noch <b>{quantity}</b> Stück da (Limit war <i>{threshold}</i>). Husch husch, ab zum Einkaufen! 🛒".to_string(),
                    },
                    MatrixMessageConfig {
                        plain: "Huhu Mama, unser {name} Vorrat geht zur Neige! Nur noch {quantity} übrig. Zeit für Nachschub!".to_string(),
                        html: "Huhu Mama! 👋 Unser <code>{name}</code> Vorrat geht zur Neige! Nur noch <b>{quantity}</b> Stück übrig. Zeit für Nachschub! 📦".to_string(),
                    }
                ],
            },
            homebox: HomeBoxConfig {
                base_url: "https://demo.homebox.software".to_string(),
                username: "foo".to_string(),
                password: Some("baz".to_string()),
                password_file: None,
                retries: default_retries(),
                items: vec![HomeBoxItemConfig {
                    id: "00000000-0000-0000-0000-000000000000".to_string(),
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
