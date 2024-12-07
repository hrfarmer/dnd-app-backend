use futures_util::StreamExt as _;

#[actix_web::get("/ws")]
pub async fn connect_ws(
    req: actix_web::HttpRequest,
    stream: actix_web::web::Payload,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    println!("Something happened");
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    actix_web::rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(actix_ws::AggregatedMessage::Text(text)) => {
                    session.text(text).await.unwrap();
                }

                Ok(actix_ws::AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }

                Ok(actix_ws::AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    Ok(res)
}
