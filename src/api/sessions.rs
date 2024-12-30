use crate::{db, AppState};
use actix_web::{get, post, web};

#[derive(serde::Deserialize)]
struct GetSessionsQuery {
    campaign_id: i32,
}

#[get("/api/get/sessions")]
pub async fn get_sessions(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
    query: web::Query<GetSessionsQuery>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if let Some(header) = req.headers().get("Authorization") {
        let access_token = &header.to_str().unwrap()[7..];
        let result = db::get_dnd_sessions(&data.db_conn, access_token, query.campaign_id)
            .await
            .map_err(|_| actix_web::error::ErrorForbidden("Unauthorized"))?;
        return Ok(actix_web::HttpResponse::Ok().body(serde_json::to_string(&result).unwrap()));
    }

    Err(actix_web::error::ErrorForbidden("No token"))
}

#[derive(serde::Deserialize)]
struct CreateSessionBody {
    campaign_id: i32,
    name: String,
}

#[post("/api/create/session")]
pub async fn create_session(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
    body: web::Json<CreateSessionBody>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if let Some(header) = req.headers().get("Authorization") {
        let access_token = &header.to_str().unwrap()[7..];
        let session =
            db::create_dnd_session(&data.db_conn, access_token, body.campaign_id, &body.name)
                .await
                .map_err(|_| actix_web::error::ErrorForbidden("Unauthorized"))?;
        return Ok(actix_web::HttpResponse::Ok().body(serde_json::to_string(&session).unwrap()));
    }
    Err(actix_web::error::ErrorForbidden("No token"))
}
