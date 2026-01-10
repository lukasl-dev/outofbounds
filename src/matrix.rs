use anyhow::Context;
use matrix_sdk::{
    Client, Room,
    ruma::{OwnedRoomId, OwnedUserId},
};

pub struct Matrix {
    client: Client,
    pub retries: u32,
}

impl Matrix {
    pub async fn new(
        user: &str,
        password: &str,
        retries: u32,
    ) -> anyhow::Result<Self> {
        let user_id = OwnedUserId::try_from(user)
            .context(format!("failed to parse matrix user id '{}'", user))?;

        for i in 0..retries {
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(i)))
                    .await;
            }

            let client = match Client::builder()
                .server_name(user_id.server_name())
                .build()
                .await
            {
                Ok(c) => c,
                Err(_) if i < retries - 1 => continue,
                Err(e) => {
                    return Err(anyhow::anyhow!(e)
                        .context("failed to build matrix client"));
                }
            };

            match client
                .matrix_auth()
                .login_username(&user_id, password)
                .send()
                .await
            {
                Ok(_) => return Ok(Self { client, retries }),
                Err(_) if i < retries - 1 => continue,
                Err(e) => {
                    return Err(anyhow::anyhow!(e).context(format!(
                        "failed to authenticate with matrix server '{}'",
                        user_id.server_name()
                    )));
                }
            }
        }

        unreachable!()
    }

    pub async fn get_room(&self, room_id_str: &str) -> anyhow::Result<Room> {
        let room_id = OwnedRoomId::try_from(room_id_str).context(format!(
            "failed to parse matrix room id '{}'",
            room_id_str
        ))?;

        for room in self.client.joined_rooms() {
            if room.room_id().eq(&room_id) {
                return Ok(room);
            }
        }

        let joined = self
            .client
            .join_room_by_id(&room_id)
            .await
            .context(format!("failed to join room '{}'", room_id_str))?;

        Ok(joined)
    }
}
