use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use anyhow;
use dnd_thing_server::{auth, config, ws, AppState};
use lazy_static::lazy_static;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    lazy_static! {
        pub static ref DISCORD_CLIENT: String =
            std::env::var("DISCORD_CLIENT").unwrap().to_string();
        pub static ref DISCORD_SECRET: String =
            std::env::var("DISCORD_SECRET").unwrap().to_string();
    }
    let client = BasicClient::new(
        ClientId::new(config::config.global.discord_client.clone()),
        Some(ClientSecret::new(
            config::config.global.discord_secret.clone(),
        )),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080/discord-token".to_string()).unwrap());

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let app_state = web::Data::new(AppState {
        auth_url: auth_url.to_string(),
        csrf_token,
        pkce_verifier: String::from(pkce_verifier.secret().clone()),
        client,
        connections: Arc::new(Mutex::new(HashMap::new())),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(auth::login_url)
            .service(auth::discord_token)
            .service(ws::ws_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
