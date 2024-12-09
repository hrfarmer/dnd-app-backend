use crate::AppState;
use actix_web::get;
use futures_util::StreamExt as _;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
enum WebsocketMessage {
    Session(DiscordUser),
    Message(String),
}
#[derive(Deserialize, Serialize, Clone)]
struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
    global_name: Option<String>,
    avatar: Option<String>,
    bot: Option<bool>,
    system: Option<bool>,
    mfa_enabled: Option<bool>,
    banner: Option<String>,
    accent_color: Option<u64>,
    locale: Option<String>,
    verified: Option<bool>,
    email: Option<String>,
    flags: Option<u64>,
    premium_type: Option<u64>,
    public_flags: Option<u64>,
}

#[get("/ws")]
async fn ws_handler(
    req: actix_web::HttpRequest,
    stream: actix_web::web::Payload,
    data: actix_web::web::Data<AppState>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let user: DiscordUser = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(token) => {
                let client = reqwest::Client::new();
                let resp = client
                    .get("https://discord.com/api/users/@me")
                    .header(reqwest::header::AUTHORIZATION, token)
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        if r.status() == StatusCode::OK {
                            r.json::<DiscordUser>().await.map_err(|_| {
                                actix_web::error::ErrorForbidden("Invalid token parsing")
                            })?
                        } else {
                            return Err(actix_web::error::ErrorForbidden("Invalid token"));
                        }
                    }
                    Err(_) => return Err(actix_web::error::ErrorForbidden("Invalid token")),
                }
            }
            Err(_) => return Err(actix_web::error::ErrorForbidden("Failed to parse token")),
        },
        None => return Err(actix_web::error::ErrorForbidden("No token provided")),
    };

    println!("{}", format!("New connection: {}", user.id.clone()));
    {
        let mut conns = data.connections.lock().unwrap();
        conns.insert(user.id.clone(), session.clone());
    }

    let session_message = WebsocketMessage::Session(user.clone());
    let _ = session.text(serde_json::to_string(&session_message)?).await;

    actix_web::rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(actix_ws::AggregatedMessage::Text(text)) => {
                    println!("New message: {}", text.to_string());
                    broadcast_message(&data, user.id.clone(), text.to_string()).await;
                }
                Ok(actix_ws::AggregatedMessage::Ping(msg)) => {
                    let _ = session.pong(&msg).await;
                }
                Err(_) => break,
                _ => {}
            }
        }
        {
            let mut conns = data.connections.lock().unwrap();
            conns.remove(&user.id.clone());
        }
    });
    Ok(res)
}
async fn broadcast_message(state: &AppState, sender_id: String, message: String) {
    let mut conns = state.connections.lock().unwrap();
    for (id, mut session) in conns.iter_mut() {
        println!("Sender id: {}, Pulled id: {}", sender_id, id);
        if *id != sender_id {
            println!("Sending to {}", id);
            let _ = session.text(message.clone()).await;
        }
    }
}
