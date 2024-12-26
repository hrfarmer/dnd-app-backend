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

pub async fn get_session(conn: &Pool<Postgres>, access_token: &str) -> Result<DiscordUser, Error> {
    let session = sqlx::query_as!(
        DiscordUser,
        "
            SELECT discord_id AS id, username, discriminator, global_name, avatar, accent_color FROM session WHERE user_id = (SELECT id FROM users WHERE access_token = $1)
        ",
        access_token
    )
    .fetch_one(conn)
    .await;

    return session;
}
