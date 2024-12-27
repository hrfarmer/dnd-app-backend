use crate::{db, UserSession};
use crate::{AppState, DiscordUser};
use actix_web::{get, web, HttpResponse, Responder};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, PkceCodeVerifier, TokenResponse};
use reqwest::StatusCode;
use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize)]
struct TokenState {
    code: String,
    state: String,
}

#[get("/login-url")]
pub async fn login_url(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(data.auth_url.clone())
}

#[get("/session")]
pub async fn session(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if let Some(header) = req.headers().get("Authorization") {
        let access_token = &header.to_str().unwrap()[7..];
        let session = db::get_session_token(&data.db_conn, access_token)
            .await
            .map_err(|_| {
                actix_web::Error::from(actix_web::error::ErrorForbidden("Failed to get session"))
            })?;
        return Ok(HttpResponse::Ok().body(serde_json::to_string(&session).unwrap()));
    }

    return Err(actix_web::error::ErrorForbidden("Failed to get session"));
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
        let res = match db::get_session_id(&data.db_conn, &user.id).await {
            Ok(r) => {
                let mut r = r;
                if r.access_token != token {
                    let tokens =
                        db::refresh_tokens(&data.db_conn, &r.access_token, &token, &refresh_token)
                            .await
                            .map_err(|_| {
                                actix_web::error::ErrorForbidden("Failed to refresh tokens")
                            })?;
                    r = UserSession {
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                        session: user,
                    };
                }
                r
            }
            Err(_) => {
                let r = db::add_user(&data.db_conn, &user, &token, &refresh_token)
                    .await
                    .map_err(|_| {
                        actix_web::Error::from(actix_web::error::ErrorForbidden(
                            "Failed to add user",
                        ))
                    })?;
                r
            }
        };

        return Ok(HttpResponse::Ok().body(serde_json::to_string(&res).unwrap()));
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
                return Err(actix_web::error::ErrorForbidden("Invaliddd token"));
            }
        }
        Err(_) => {
            return Err(actix_web::error::ErrorForbidden("Invalid token"));
        }
    };

    Ok(user)
}
