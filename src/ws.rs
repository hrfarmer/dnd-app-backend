use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{AppState, DiscordUser};
use actix_web::get;
use actix_ws::{AggregatedMessage, Message};
use futures_util::{future, StreamExt as _};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::{pin, time::interval};

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
enum WebsocketMessage {
    Session(DiscordUser),
    ConnectedUsers(HashMap<String, DiscordUser>),
    Message(String),
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

    {
        let mut sessions = data.sessions.lock().unwrap();
        sessions.insert(user.id.clone(), user.clone());

        let message = WebsocketMessage::ConnectedUsers(sessions.clone());
        let _ = session.text(serde_json::to_string(&message)?).await;
    }

    // ping variables
    const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
    const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    actix_web::rt::spawn(async move {
        let reason = loop {
            let tick = interval.tick();
            pin!(tick);

            match future::select(stream.next(), tick).await {
                future::Either::Left((Some(Ok(msg)), _)) => {
                    match msg {
                        AggregatedMessage::Text(text) => {
                            let time = chrono::Utc::now().format("%H:%M:%S").to_string();
                            println!(
                                "{:} msg from {}: {:?}",
                                time,
                                user.username,
                                text.to_string()
                            );
                            broadcast_message(&data, user.id.clone(), text.to_string()).await;
                        }

                        // binary not used
                        AggregatedMessage::Binary(_) => {
                            continue;
                        }

                        AggregatedMessage::Close(reason) => {
                            break reason;
                        }

                        AggregatedMessage::Ping(bytes) => {
                            last_heartbeat = Instant::now();
                            let _ = session.pong(&bytes).await;
                        }

                        AggregatedMessage::Pong(_) => {
                            last_heartbeat = Instant::now();
                        }
                    }
                }

                // client WebSocket stream error
                future::Either::Left((Some(Err(err)), _)) => {
                    eprintln!("{}", err);
                    break None;
                }

                // client WebSocket stream ended
                future::Either::Left((None, _)) => break None,

                // heartbeat ticked
                future::Either::Right((_inst, _)) => {
                    if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                        println!("Client didn't respond to heartbeat, disconnecting");

                        break None;
                    }

                    let _ = session.ping(b"").await;
                }
            }
        };

        // disconnect and remove user
        let _ = session.close(reason).await;
        println!("User {} disconnecting", user.id);

        let mut conns = data.connections.lock().unwrap();
        let mut sessions = data.sessions.lock().unwrap();
        conns.remove(&user.id.clone());
        sessions.remove(&user.id.clone());
    });
    Ok(res)
}

async fn broadcast_message(state: &AppState, sender_id: String, message: String) {
    let mut conns = state.connections.lock().unwrap();
    for (id, session) in conns.iter_mut() {
        if *id != sender_id {
            println!("Sending to {}", id);
            let _ = session.text(message.clone()).await;
        }
    }
}
