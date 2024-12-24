use actix_ws::Session;
use oauth2::basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenResponse, BasicTokenType,
};
use oauth2::{Client, CsrfToken, StandardRevocableToken};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod auth;
pub mod config;
pub mod db;
pub mod ws;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
    global_name: Option<String>,
    avatar: Option<String>,
    bot: Option<bool>,
    system: Option<bool>,
    mfa_enabled: Option<bool>,
    banner: Option<String>,
    accent_color: Option<i32>,
    locale: Option<String>,
    verified: Option<bool>,
    email: Option<String>,
    flags: Option<i32>,
    premium_type: Option<i32>,
    public_flags: Option<i32>,
}

pub struct AppState {
    pub auth_url: String,
    pub csrf_token: CsrfToken,
    pub pkce_verifier: String,
    pub client: Client<
        BasicErrorResponse,
        BasicTokenResponse,
        BasicTokenType,
        BasicTokenIntrospectionResponse,
        StandardRevocableToken,
        BasicRevocationErrorResponse,
    >,
    pub connections: Arc<Mutex<HashMap<String, Session>>>,
    pub sessions: Arc<Mutex<HashMap<String, DiscordUser>>>,
    pub db_conn: Pool<Postgres>,
}
