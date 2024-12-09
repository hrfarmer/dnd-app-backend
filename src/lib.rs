use actix_ws::Session;
use oauth2::basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenResponse, BasicTokenType,
};
use oauth2::{Client, CsrfToken, StandardRevocableToken};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod auth;
pub mod config;
pub mod ws;

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
}
