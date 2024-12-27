use oauth2::basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenResponse, BasicTokenType,
};
use oauth2::{Client, StandardRevocableToken};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod auth;
pub mod config;
pub mod db;
pub mod ws;

#[derive(Serialize, Deserialize)]
pub struct UserSession {
    access_token: String,
    refresh_token: String,
    session: DiscordUser,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
    global_name: Option<String>,
    avatar: Option<String>,
    accent_color: Option<i32>,
}

pub struct AppState {
    pub client: Client<
        BasicErrorResponse,
        BasicTokenResponse,
        BasicTokenType,
        BasicTokenIntrospectionResponse,
        StandardRevocableToken,
        BasicRevocationErrorResponse,
    >,
    pub connections: Arc<Mutex<HashMap<String, actix_ws::Session>>>,
    pub sessions: Arc<Mutex<HashMap<String, DiscordUser>>>,
    pub pending_logins: Arc<Mutex<HashMap<String, actix::Addr<ws::LoginActor>>>>,
    pub db_conn: Pool<Postgres>,
}
