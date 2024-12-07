use crate::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, PkceCodeVerifier, TokenResponse};
use serde::Deserialize;

#[get("/login-url")]
pub async fn login_url(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(data.auth_url.clone())
}

#[derive(Deserialize)]
struct TokenState {
    code: String,
    state: String,
}

#[get("/discord-token")]
pub async fn discord_token(
    token: web::Query<TokenState>,
    data: web::Data<AppState>,
) -> impl Responder {
    let token_result = data
        .client
        .exchange_code(AuthorizationCode::new(token.code.clone()))
        .set_pkce_verifier(PkceCodeVerifier::new(data.pkce_verifier.clone()))
        .request_async(async_http_client)
        .await;

    match token_result {
        Ok(t) => HttpResponse::Ok().body(format!(
            "Paste this code into the program: {}",
            t.access_token().secret().to_string().clone()
        )),
        Err(_) => HttpResponse::Forbidden().body("Failed to authenticate"),
    }
}
