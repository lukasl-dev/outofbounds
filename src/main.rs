use anyhow::Context;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::config::Config;
use crate::homebox::HomeBox;
use crate::matrix::Matrix;

mod config;
mod homebox;
mod matrix;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::load("config.toml")?;

    let matrix = Matrix::new(&cfg.matrix.user, &cfg.matrix.password).await?;
    println!("Logged into matrix!");

    let homebox = HomeBox::new(
        &cfg.homebox.base_url,
        &cfg.homebox.username,
        &cfg.homebox.password,
    )
    .await?;
    println!("Logged into homebox!");

    let room = matrix.get_room(&cfg.matrix.room_id).await?;

    for cfg_item in cfg.homebox.items {
        let items = homebox
            .get_item_by_asset_id(&cfg_item.asset_id)
            .await
            .context(format!("failed to get item '{}'", cfg_item.asset_id))?;

        for item in items {
            if item.quantity <= cfg_item.threshold {
                for template in &cfg.matrix.messages {
                    let plain = template
                        .plain
                        .replace("{name}", &item.name)
                        .replace("{quantity}", &item.quantity.to_string())
                        .replace("{threshold}", &cfg_item.threshold.to_string())
                        .replace("{asset_id}", &item.asset_id);

                    let html = template
                        .html
                        .replace("{name}", &item.name)
                        .replace("{quantity}", &item.quantity.to_string())
                        .replace("{threshold}", &cfg_item.threshold.to_string())
                        .replace("{asset_id}", &item.asset_id);

                    room.send(RoomMessageEventContent::text_html(plain, html))
                        .await
                        .context(format!(
                            "failed to send message to room '{}'",
                            room.room_id()
                        ))?;
                }
            } else {
                println!(
                    "Enough ({} > {}) of item '{}'",
                    item.quantity, cfg_item.threshold, item.asset_id
                );
            }
        }
    }

    Ok(())
}
