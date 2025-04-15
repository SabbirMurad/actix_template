use chrono::Utc;
use mongodb::bson::doc;
use crate::{builtins::jwt, schema::Account};
use crate::BuiltIn::mongo::MongoDB;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{ web, Error, HttpResponse };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostData {
    user_id: String,
    validation_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStruct {
    uuid: String,
    email_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfoStruct {
    uuid: String,
    full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Payload {
    access_token: String,
    refresh_token: String,
    user_id: String,
    role: Account::AccountRole
}

pub async fn task(form_data: web::Json<PostData>) -> Result<HttpResponse, Error> {
    let post_data = sanitize(&form_data);

    if let Err(res) = check_empty_fields(&post_data) {
        return Ok(Response::bad_request(&res));
    }

    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    if let Err(error) = session.start_transaction(None).await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    //checking if user exist
    let collection = db.collection::<Account::AccountCore>("account_core");
    let result = collection.find_one_with_session(
        doc!{"uuid": &post_data.user_id},
        None,
        &mut session
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::not_found("User not found"));
    }

    let account_core = option.unwrap();
    if account_core.email_verified {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::forbidden("User already verified"));
    }

    //check if validation code match
    let collection = db.collection::
    <Account::AccountVerificationRequest>("account_verification_request");
    let result = collection.find_one_with_session(
        doc!{"user_id": &post_data.user_id},
        None,
        &mut session
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::not_found("Validation request not found"));
    }

    let request = option.unwrap();
    if request.expires_at < Utc::now().timestamp_millis() {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::forbidden("Validation code expired"));
    }

    if request.validation_code != post_data.validation_code {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::forbidden("Validation code incorrect"));
    }

    //make user verified
    let collection = db.collection::<Account::AccountCore>("account_core");
    let result = collection.update_one_with_session(
        doc!{ "uuid": &post_data.user_id },
        doc!{"$set": { "email_verified": true }},
        None,
        &mut session
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let count = result.unwrap().modified_count;
    if count == 0 {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::not_found("User not found"));
    }

    //delete validation request
    let collection = db.collection
    ::<Account::AccountVerificationRequest>("account_verification_request");
    let result = collection.delete_many_with_session(
        doc!{"user_id": &post_data.user_id},
        None,
        &mut session
    ).await;

    if let Err(error) = result {
      log::error!("{:?}", error);
      session.abort_transaction().await.ok().unwrap();
      return Ok(Response::internal_server_error(&error.to_string()));
    }

    //create account social
    let empty_vec = Vec::new();
    let now = Utc::now().timestamp_millis();
    let account_social = Account::AccountSocial {
        uuid: account_core.uuid.clone(),
        liked_by: empty_vec.clone(),
        followers: empty_vec.clone(),
        following: empty_vec.clone(),
        friends: empty_vec.clone(),
        blocked: empty_vec.clone(),

        like_count: 0,
        follower_count: 0, 
        following_count: 0,
        friend_count: 0,
        blocked_count: 0,

        modified_at: now,
    };

    let collection = db.collection
    ::<Account::AccountSocial>("account_social");
    let result = collection.insert_one_with_session(
        account_social,
        None,
        &mut session
    ).await;

    if let Err(error) = result {
      log::error!("{:?}", error);
      session.abort_transaction().await.ok().unwrap();
      return Ok(Response::internal_server_error(&error.to_string()));
    }

    //create account status
    let account_status = Account::AccountStatus {
        uuid: account_core.uuid.clone(),
        online: false,
        last_seen: now,
        fcm_tokens: empty_vec.clone(),
    };
    let collection = db.collection
    ::<Account::AccountStatus>("account_status");
    let result = collection.insert_one_with_session(
        account_status,
        None,
        &mut session
    ).await;

    if let Err(error) = result {
      log::error!("{:?}", error);
      session.abort_transaction().await.ok().unwrap();
      return Ok(Response::internal_server_error(&error.to_string()));
    }

    //create account notification settings
    let account_notification_settings = Account::AccountNotificationSettings {
        uuid: account_core.uuid.clone(),
        friend_request_notification: true,
        following_notification: true,
        appreciation_notification: true,
        comment_notification: true,
        tag_notification: true,
        modified_at: now,
    };

    let collection = db.collection
    ::<Account::AccountNotificationSettings>("account_notification_settings");
    let result = collection.insert_one_with_session(
        account_notification_settings,
        None,
        &mut session
    ).await;

    if let Err(error) = result {
      log::error!("{:?}", error);
      session.abort_transaction().await.ok().unwrap();
      return Ok(Response::internal_server_error(&error.to_string()));
    }

    // getting access token
    let access_token = jwt::access_token::generate_default(
        &account_core.uuid,
        account_core.role.clone(),
    );

    // getting refresh token
    let result = jwt::refresh_token::new(&account_core.uuid);
    if let Err(error) = result {
        session.abort_transaction().await.ok().unwrap();
        log::error!("{:?}", error); 
        return Ok(Response::internal_server_error(&error));
    }

    let refresh_token = result.unwrap();

    /* DATABASE ACID COMMIT */
    if let Err(error) = session.commit_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let payload = Payload {
        access_token,
        refresh_token,
        user_id: account_core.uuid,
        role: account_core.role
    };
    Ok(HttpResponse::Ok().content_type("application/json").json(payload))
}

fn sanitize(form_data: &PostData) -> PostData {
  let mut form = form_data.clone();
  form.user_id = form.user_id.trim().to_string();
  form.validation_code = form.validation_code.trim().to_string();

  form
}

fn check_empty_fields(form_data: &PostData) -> Result<(), String> {
  if form_data.user_id.len() == 0 {
    Err("User id required".to_string())
  }
  else if form_data.validation_code.len() == 0 {
    Err("Validation code required".to_string())
  }
  else {
    Ok(())
  }
}