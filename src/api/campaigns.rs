use actix_web::{get, post, web};
use serde::Deserialize;

use crate::{db, AppState};

#[derive(Deserialize)]
struct CreateCampaignBody {
    name: String,
}

#[post("/api/create/campaign")]
pub async fn create_campaign(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
    body: web::Json<CreateCampaignBody>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if let Some(header) = req.headers().get("Authorization") {
        let access_token = &header.to_str().unwrap()[7..];

        let res = db::create_dnd_campaign(&data.db_conn, access_token, &body.name)
            .await
            .map_err(|_| actix_web::error::ErrorForbidden("Failed to create session"))?;

        return Ok(actix_web::HttpResponse::Ok().body(serde_json::to_string(&res).unwrap()));
    }

    Err(actix_web::error::ErrorForbidden("No token"))
}

#[get("/api/get/campaigns")]
pub async fn get_campaigns(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if let Some(header) = req.headers().get("Authorization") {
        let access_token = &header.to_str().unwrap()[7..];

        let res = db::get_dnd_campaigns(&data.db_conn, access_token)
            .await
            .map_err(|_| actix_web::error::ErrorForbidden("Failed to get campaigns"))?;

        return Ok(actix_web::HttpResponse::Ok().body(serde_json::to_string(&res).unwrap()));
    }

    Err(actix_web::error::ErrorForbidden("No token"))
}

#[derive(Deserialize)]
struct JoinCampaignBody {
    invite: String,
}

#[post("/api/join/campaign")]
pub async fn join_campaign(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
    body: web::Json<JoinCampaignBody>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if let Some(header) = req.headers().get("Authorization") {
        let access_token = &header.to_str().unwrap()[7..];

        let res = db::join_dnd_campaign(&data.db_conn, access_token, &body.invite).await;
        match res {
            Ok(_) => return Ok(actix_web::HttpResponse::Ok().body("Joined campaign")),
            Err(e) => {
                return Err(actix_web::error::ErrorForbidden(format!(
                    "Invite has no more uses or something else went wrong: {}",
                    e
                )))
            }
        };
    };

    Err(actix_web::error::ErrorForbidden("No token"))
}
