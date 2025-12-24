use anyhow::Context;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use rand::seq::SliceRandom;

use crate::config::Config;
use crate::homebox::HomeBox;
use crate::matrix::Matrix;

mod config;
mod homebox;
mod matrix;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = std::env::args().nth(1).unwrap_or_else(|| "config.toml".to_string());
    let cfg = Config::load(&config_path)?;

    let matrix_password = cfg.matrix.resolve_password()?;
    let matrix = Matrix::new(&cfg.matrix.user, &matrix_password).await?;
    println!("Logged into matrix!");

    let homebox_password = cfg.homebox.resolve_password()?;
    let homebox = HomeBox::new(
        &cfg.homebox.base_url,
        &cfg.homebox.username,
        &homebox_password,
    )
    .await?;
    println!("Logged into homebox!");

    let room = matrix.get_room(&cfg.matrix.room_id).await?;

    for cfg_item in cfg.homebox.items {
        let item = homebox
            .get_item(&cfg_item.id)
            .await
            .context(format!("failed to get item '{}'", cfg_item.id))?;

        if item.quantity <= cfg_item.threshold {
            if let Some(template) = cfg.matrix.messages.choose(&mut rand::thread_rng()) {
                let plain = template
                    .plain
                    .replace("{name}", &item.name)
                    .replace("{quantity}", &item.quantity.to_string())
                    .replace("{threshold}", &cfg_item.threshold.to_string())
                    .replace("{asset_id}", &item.asset_id)
                    .replace("{id}", &item.id);

                let html = template
                    .html
                    .replace("{name}", &item.name)
                    .replace("{quantity}", &item.quantity.to_string())
                    .replace("{threshold}", &cfg_item.threshold.to_string())
                    .replace("{asset_id}", &item.asset_id)
                    .replace("{id}", &item.id);

                room.send(RoomMessageEventContent::text_html(plain, html))
                    .await
                    .context(format!(
                        "failed to send message to room '{}'",
                        room.room_id()
                    ))?;
            }
        } else {
            println!(
                "Enough ({} > {}) of item '{}' ({})",
                item.quantity, cfg_item.threshold, item.name, item.id
            );
        }
    }

    Ok(())
}
