use crate::BuiltIn::jwt;
use mongodb::bson::doc;
use crate::schema::Account;
use actix_session::Session;
use crate::builtins::mongo::MongoDB;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use crate::utils::header::authorization_token;
use actix_web::{web, Error, HttpResponse, HttpRequest};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PostData { fcm_token: String }

pub async fn task(req: HttpRequest, form_data: web::Json<PostData>, actix_session: Session) -> Result<HttpResponse, Error> {
    //getting access token out
    let access_token = match authorization_token(req.headers()) {
        Ok(token) => token,
        Err(err) => {
            return Ok(Response::bad_request(err));
        }
    };

    //validate user id
    let user_id = match jwt::access_token::verify(
        &access_token,
        jwt::Key::Local
    ) {
        Ok(claims) => claims.sub,
        Err(err) => {
            return Ok(Response::unauthorized(&err));
        }
    };
    
    let db = MongoDB.connect();
    let collection = db.collection::<Account::AccountStatus>("account_status");
    let result = collection.update_one(
        doc!{"uuid": &user_id},
        doc!{"$pull": {"fcm_tokens": &form_data.fcm_token}},
        None,
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    actix_session.purge();
    Ok(HttpResponse::Ok().content_type("application/json").json(
        Response { message: "Successfully Signed Out".to_string() }
    ))
}