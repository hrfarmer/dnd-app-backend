use crate::AppState;
use actix_web::get;
use actix_ws::Session;
use futures_util::StreamExt as _;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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

    let conn_id = Uuid::new_v4();

    println!("{}", format!("New connection: {}", conn_id));

    {
        let mut conns = data.connections.lock().unwrap();
        conns.insert(conn_id, session.clone());
    }

    actix_web::rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(actix_ws::AggregatedMessage::Text(text)) => {
                    println!("New message: {}", text.to_string());
                    broadcast_message(&data, &conn_id, text.to_string()).await;
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
            conns.remove(&conn_id);
        }
    });

    Ok(res)
}

async fn broadcast_message(state: &AppState, sender_id: &Uuid, message: String) {
    let mut conns = state.connections.lock().unwrap();
    for (id, mut session) in conns.iter_mut() {
        println!("Sender id: {}, Pulled id: {}", sender_id, id);
        if id != sender_id {
            println!("Sending to {}", id);
            let _ = session.text(message.clone()).await;
        }
    }
}
