use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dnd_thing_server::{auth, config, ws, AppState};
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = BasicClient::new(
        ClientId::new(config::config.global.discord_client.clone()),
        Some(ClientSecret::new(
            config::config.global.discord_secret.clone(),
        )),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080/login".to_string()).unwrap());

    let conn = sqlx::postgres::PgPool::connect(config::config.global.database_url.as_str())
        .await
        .unwrap();

    let app_state = web::Data::new(AppState {
        client,
        connections: Arc::new(Mutex::new(HashMap::new())),
        sessions: Arc::new(Mutex::new(HashMap::new())),
        pending_logins: Arc::new(Mutex::new(HashMap::new())),
        db_conn: conn,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(auth::discord_token)
            .service(auth::session)
            .service(ws::ws_handler)
            .service(ws::ws_login)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
