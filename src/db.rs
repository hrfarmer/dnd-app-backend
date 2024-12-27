use sqlx::{Error, Pool, Postgres};

use crate::{DiscordUser, UserSession};

pub async fn add_user(
    conn: &Pool<Postgres>,
    user: &DiscordUser,
    access_token: &str,
    refresh_token: &str,
) -> Result<UserSession, Error> {
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
                return Ok(UserSession{access_token: access_token.to_string(), refresh_token: refresh_token.to_string(), session: user.clone()});
            }
            Err(err) => {
                dbg!(&err);
                return Err(err);
            }
        };
}

pub struct AccessTokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn get_session_token(
    conn: &Pool<Postgres>,
    access_token: &str,
) -> Result<UserSession, Error> {
    let tokens = sqlx::query_as!(
        AccessTokens,
        "SELECT access_token, refresh_token FROM users WHERE access_token = $1",
        access_token
    )
    .fetch_one(conn)
    .await?;

    let session = sqlx::query_as!(
        DiscordUser,
        "
            SELECT discord_id AS id, username, discriminator, global_name, avatar, accent_color FROM session WHERE user_id = (SELECT id FROM users WHERE access_token = $1)
        ",
        access_token
    )
    .fetch_one(conn)
    .await?;

    return Ok(UserSession {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        session,
    });
}

pub async fn get_session_id(conn: &Pool<Postgres>, id: &str) -> Result<UserSession, Error> {
    let session = sqlx::query_as!(
        DiscordUser,
        "
            SELECT discord_id AS id, username, discriminator, global_name, avatar, accent_color FROM session WHERE discord_id = $1
        ",
        id
    )
    .fetch_one(conn)
    .await?;

    let tokens = sqlx::query_as!(
        AccessTokens,
        "SELECT access_token, refresh_token FROM users WHERE EXISTS (SELECT user_id FROM session WHERE discord_id = $1)",
        id
    )
    .fetch_one(conn)
    .await?;

    return Ok(UserSession {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        session,
    });
}

pub async fn refresh_tokens(
    conn: &Pool<Postgres>,
    access_token: &str,
    new_access_token: &str,
    new_refresh_token: &str,
) -> Result<AccessTokens, Error> {
    sqlx::query!(
        "UPDATE users SET access_token = $1, refresh_token = $2 WHERE access_token = $3",
        new_access_token,
        new_refresh_token,
        access_token
    )
    .execute(conn)
    .await?;

    Ok(AccessTokens {
        access_token: new_access_token.to_string(),
        refresh_token: new_refresh_token.to_string(),
    })
}
