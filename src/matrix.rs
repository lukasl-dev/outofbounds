use anyhow::Context;
use matrix_sdk::{
    Client, Room,
    ruma::{OwnedRoomId, OwnedUserId},
};

pub struct Matrix {
    client: Client,
}

impl Matrix {
    pub async fn new(user: &str, password: &str) -> anyhow::Result<Self> {
        let user_id = OwnedUserId::try_from(user)
            .context(format!("failed to parse matrix user id '{}'", user))?;

        let client = Client::builder()
            .server_name(user_id.server_name())
            .build()
            .await
            .context("failed to build matrix client")?;

        client
            .matrix_auth()
            .login_username(&user_id, password)
            .send()
            .await
            .context(format!(
                "failed to authenticate with matrix server '{}'",
                user_id.server_name()
            ))?;

        Ok(Self { client })
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
