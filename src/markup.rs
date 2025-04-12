use tera::{Tera, Context};
use actix_web::{web, error, Error, HttpRequest, HttpResponse};
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;

pub async fn home(template: web::Data<Tera>) -> Result<HttpResponse, Error> {
  let res_data = template.render(
    "public/home.html",
    &Context::new()
  )
  .map_err(|e|error::ErrorInternalServerError(e))?;
  
  Ok(HttpResponse::Ok().content_type("text/html").body(res_data))
}