use crate::db;
use crate::{AppState, DiscordUser};
use actix_web::{get, web, HttpResponse, Responder};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, PkceCodeVerifier, TokenResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Deserialize)]
struct TokenState {
    code: String,
    state: String,
}

#[derive(Serialize, Deserialize)]
struct Session {
    access_token: String,
    refresh_token: String,
    session: DiscordUser,
}

#[get("/login-url")]
pub async fn login_url(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(data.auth_url.clone())
}

#[get("/login")]
pub async fn discord_token(
    token: web::Query<TokenState>,
    data: web::Data<AppState>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let token_result = data
        .client
        .exchange_code(AuthorizationCode::new(token.code.clone()))
        .set_pkce_verifier(PkceCodeVerifier::new(data.pkce_verifier.clone()))
        .request_async(async_http_client)
        .await;

    if let Ok(t) = token_result {
        let token = t.access_token().secret().to_string().clone();
        let refresh_token = t.refresh_token().unwrap().secret().to_string().clone();

        let user = get_discord_user(token.to_string()).await?;
        if let Ok(res) = db::get_session(&data.db_conn, &token).await {
            if res.id != user.id {
                let _ = db::add_user(&data.db_conn, &user, &token, &refresh_token)
                    .await
                    .map_err(|_| {
                        actix_web::Error::from(actix_web::error::ErrorForbidden(
                            "Failed to add user",
                        ))
                    })?;
            }

            return Ok(HttpResponse::Ok().body(
                serde_json::to_string(&Session {
                    access_token: token,
                    refresh_token,
                    session: user,
                })
                .unwrap(),
            ));
        }
    }

    Err(actix_web::error::ErrorForbidden("Failed to log in"))
}

pub async fn get_discord_user(token: String) -> Result<DiscordUser, actix_web::Error> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://discord.com/api/users/@me")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await;

    let user = match resp {
        Ok(r) => {
            if r.status() == StatusCode::OK {
                r.json::<DiscordUser>()
                    .await
                    .map_err(|_| actix_web::error::ErrorForbidden("Invalid token parsing"))?
            } else {
                return Err(actix_web::error::ErrorForbidden("Invalid token"));
            }
        }
        Err(_) => {
            return Err(actix_web::error::ErrorForbidden("Invalid token"));
        }
    };

    Ok(user)
}
