use crate::BuiltIns::jwt;
use futures::future::{ready, Ready};
use crate::schema::Account::AccountRole;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};

#[derive(Debug, Clone)]
pub enum AccessRequirement {
    None,
    AnyToken,
    Role(AccountRole),
    AnyOf(Vec<AccountRole>),
}

#[derive(Debug)]
pub struct RequireAccess {
    pub user_id: Option<String>,
    pub role: Option<AccountRole>,
}

impl FromRequest for RequireAccess {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let config = req
            .app_data::<AccessRequirement>()
            .cloned()
            .unwrap_or(AccessRequirement::None);

        match config {
            AccessRequirement::None => {
                return ready(Ok(Self { user_id: None, role: None }));
            }
            _ => {
                let auth_header = req
                    .headers()
                    .get("Authorization")
                    .and_then(|h| h.to_str().ok());

                if auth_header.is_none() {
                    return ready(Err(actix_web::error::ErrorUnauthorized(
                        "Missing access token"
                    )));
                }

                let token = auth_header.unwrap()
                .trim_start_matches("Bearer ").to_string();

                // Validating access token
                let result = jwt::access_token::verify(
                    &token,
                    jwt::Key::Local
                );

                if let Err(err) = result {
                    log::error!("{:?}", err);
                    return ready(Err(actix_web::error::ErrorUnauthorized(
                        "Invalid access token"
                    )));
                }

                let claims = result.unwrap();

                let pass = match &config {
                    AccessRequirement::AnyToken => true,
                    AccessRequirement::Role(r) => &claims.role == r,
                    AccessRequirement::AnyOf(roles) => roles.contains(&claims.role),
                    _ => false,
                };

                if pass {
                    ready(Ok(Self {
                        user_id: Some(claims.sub),
                        role: Some(claims.role),
                    }))
                } else {
                    ready(Err(actix_web::error::ErrorForbidden(
                        "Not authorized to perform this action"
                    )))
                }
            }
        }
    }
}
