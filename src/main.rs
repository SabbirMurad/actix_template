use std::{ env, io };
use actix::Actor;
use listenfd::ListenFd;
use tera::{ Tera, Context };
use dotenv::dotenv as App_env;
use actix_files as StaticResource;
use futures_util::future::{self, Either, FutureExt};
use actix_session::{ SessionMiddleware, storage::RedisActorSessionStore };
use actix_session::config::{ PersistentSession, TtlExtensionPolicy, CookieContentSecurity };
use actix_web::{ web, dev, http, error, middleware as ActixMiddleware, App, dev::Service, cookie::SameSite, cookie::Key, cookie::time::Duration, HttpResponse, HttpServer };

mod builtins;
use builtins as BuiltIns;

mod schema;
use schema as Schema;

mod middleware;
use middleware as Middleware;

mod utils;
use utils as Utils;

mod routes;
use routes as Routes;

mod integrations;
use integrations as Integrations;

mod handler;
use handler as Handler;

mod markup;
use markup as Markup;

fn main() {
    /*
      Loads environment variables from `.env` file to `std::env`
    */ 
    App_env().ok();

    /*
      Use with log::info!() | log::warn!() | log::error!()
    */
    let _logger = BuiltIns::logger::init();

    /*
      RUNS CORN JOB FOR YOUR PROJECT
      Remove the following code block if you are not using this feature.
    */
    /* tokio::spawn(async move {
        use tokio::time::{self, Duration};
        let mut interval = time::interval(Duration::from_secs(60));   
        loop {
            interval.tick().await;

            /*
                Execute your desired CORN routines here.
            */ 
            BuiltIn::cron::greet().await;
        }
        });
    */
}