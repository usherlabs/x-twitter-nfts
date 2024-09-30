#[macro_use]
extern crate rocket;
extern crate helper;

pub mod handler;
pub mod models;

use dotenvy::dotenv;
use handler::{catcher_handler::*, open_api_handler::open_api_specification, tweet::{mint_tweet_request, tweet_contract_call}};

use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[launch]
fn rocket() -> _ {
    dotenv().expect("Error occured when loading .env");

    env::var("THIRDWEB_CLIENT_ID").expect("THIRDWEB_CLIENT_ID must be set");
    env::var("TWEET_BEARER").expect("TWEET_BEARER must be set");


    tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| {
                return format!("{}=debug,rocket=info", env!("CARGO_CRATE_NAME")).into();
        })),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

    rocket::build()
    .configure(rocket::Config::figment().merge(("port", 8007)))
    .mount("/.well-known", routes![open_api_specification])
    .mount("/api", routes![mint_tweet_request,tweet_contract_call])
    .register("/", catchers![unprocessable_entity_catcher,not_found])
}
