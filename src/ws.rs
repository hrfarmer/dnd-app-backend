use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    time::{Duration, Instant},
};

use crate::{auth::get_discord_user, AppState, DiscordUser};
use actix::{Actor, ActorContext};
use actix_web::get;
use actix_ws::{AggregatedMessage, CloseReason, Session};
use futures_util::{future, StreamExt as _};
use oauth2::{CsrfToken, Scope};
use serde::{Deserialize, Serialize};
use tokio::{pin, time::interval};

#[derive(Deserialize, Serialize, Clone)]
struct ChatMessage {
    author: String,
    content: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
enum WebsocketMessage {
    Session(DiscordUser),
    ConnectedUsers(HashMap<String, DiscordUser>),
    Message(ChatMessage),
    Disconnect(String),
}

// Actor information for login websocket
pub struct LoginActor {
    session: Session,
}

impl LoginActor {
    fn new(session: Session) -> Self {
        LoginActor { session }
    }
}

impl Actor for LoginActor {
    type Context = actix::Context<Self>;
}

#[derive(actix::Message, Deserialize)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct LoginPayload {
    pub payload: crate::UserSession,
}

impl actix::Handler<LoginPayload> for LoginActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: LoginPayload, ctx: &mut actix::Context<Self>) -> Self::Result {
        let mut session: Session = self.session.clone();
        actix_web::rt::spawn(async move {
            let _ = session
                .text(serde_json::to_string(&msg.payload).unwrap())
                .await
                .map_err(|_| Error::new(ErrorKind::Other, "Failed to send"));

            let _ = session
                .close(Some(CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some(String::from("Sent login")),
                }))
                .await;
        });

        ctx.stop();

        Ok(true)
    }
}

#[get("/ws-login")]
async fn ws_login(
    req: actix_web::HttpRequest,
    stream: actix_web::web::Payload,
    data: actix_web::web::Data<AppState>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    println!("Client connected to login");
    let (res, mut session, _) = actix_ws::handle(&req, stream)?;

    let state_value = uuid::Uuid::new_v4();

    let (auth_url, _) = data
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_extra_param("state", &state_value.to_string())
        .url();

    let _ = session.text(auth_url.to_string()).await;

    let addr = LoginActor::new(session).start();
    {
        let mut pending_logins = data.pending_logins.lock().unwrap();
        pending_logins.insert(state_value.to_string(), addr);
    }

    Ok(res)
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
            Ok(token) => match get_discord_user(token[7..].to_string()).await {
                Ok(user) => user,
                Err(err) => return Err(err),
            },
            Err(_) => return Err(actix_web::error::ErrorForbidden("Failed to parse token")),
        },
        None => return Err(actix_web::error::ErrorForbidden("No token provided")),
    };

    println!("{}", format!("New connection: {}", user.id.clone()));

    let session_message = WebsocketMessage::Session(user.clone());
    let _ = session.text(serde_json::to_string(&session_message)?).await;

    {
        let mut conns = data.connections.lock().unwrap();
        conns.insert(user.id.clone(), session.clone());

        let mut sessions = data.sessions.lock().unwrap();
        sessions.insert(user.id.clone(), user.clone());

        let message = WebsocketMessage::ConnectedUsers(sessions.clone());

        for (_, session) in conns.iter_mut() {
            let _ = session.text(serde_json::to_string(&message)?).await;
        }
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

                            if let Ok(message) = serde_json::from_str::<WebsocketMessage>(&text) {
                                if let WebsocketMessage::Disconnect(reason) = message {
                                    break Some(CloseReason {
                                        code: actix_ws::CloseCode::Normal,
                                        description: Some(reason),
                                    });
                                }
                            }
                            handle_message(&data, user.id.clone(), text.to_string()).await;
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

        let message = WebsocketMessage::ConnectedUsers(sessions.clone());

        for (_, session) in conns.iter_mut() {
            let _ = session.text(serde_json::to_string(&message).unwrap()).await;
        }
    });
    Ok(res)
}

async fn handle_message(state: &AppState, sender_id: String, message: String) {
    let mut conns = state.connections.lock().unwrap();
    for (id, session) in conns.iter_mut() {
        if *id != sender_id {
            // just one message type for now, will handle more message types later
            let _ = session
                .text(
                    serde_json::to_string(&WebsocketMessage::Message(ChatMessage {
                        author: sender_id.clone(),
                        content: message.clone(),
                    }))
                    .unwrap(),
                )
                .await;
        }
    }
}
