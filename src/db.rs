use sqlx::{Error, Pool, Postgres};

use crate::DiscordUser;

pub async fn add_user(
    conn: &Pool<Postgres>,
    user: &DiscordUser,
    access_token: &str,
    refresh_token: &str,
) -> Result<bool, Error> {
    let user_response = sqlx::query!(
        "INSERT INTO users (username, access_token, refresh_token) VALUES ($1, $2, $3) RETURNING id",
        user.username,
        access_token,
        refresh_token
    )
    .fetch_one(conn)
    .await?;

    match sqlx::query!("INSERT INTO session (user_id, discord_id, username, discriminator, global_name, avatar, accent_color) VALUES ($1, $2, $3, $4, $5, $6, $7)", user_response.id, user.id, user.username, user.discriminator, user.global_name.as_ref().unwrap_or(&user.username), user.avatar, user.accent_color).execute(conn).await {
            Ok(_) => {
                return Ok(true);
            }
            Err(err) => {
                dbg!(&err);
                return Err(err);
            }
        };
}
