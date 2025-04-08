#[macro_use]
extern crate rocket;

pub mod handler;
pub mod helper;
pub mod models;

use handler::{
    catcher_handler::*,
    open_api_handler::open_api_specification,
    tweet::{mint_tweet_request, tweet_contract_call, tweet_contract_cancel_call},
};

use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[launch]
fn rocket() -> _ {
    let host_url = env::var("HOST_URL");

    // For Production host_url must be set has environment Variable to prevent loading plugin.env that will not exist
    if host_url.is_err() {
        dotenvy::from_filename("plugin.env").expect("Error occurred when loading plugin.env");
    }
    println!(
        "Loaded plugin.env with account_id: {}",
        env::var("ACCOUNT_ID").unwrap()
    );

    //Check if required Variable are set
    env::var("THIRDWEB_CLIENT_ID").expect("THIRDWEB_CLIENT_ID must be set");
    env::var("TWEET_BEARER").expect("TWEET_BEARER must be set");
    env::var("ACCOUNT_ID").expect("ACCOUNT_ID must be set");

    // Initialize tracing
    tracing_subscriber::registry()
        // Set up an environment filter based on the RUST_LOG environment variable
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG")
                // If RUST_LOG is not set, default to "crate_name=debug,rocket=info"
                .unwrap_or_else(|_| {
                    return format!("{}=debug,rocket=info", env!("CARGO_CRATE_NAME")).into();
                }),
        ))
        // Add the fmt layer for pretty-printing logs
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        // Initialize the tracing subscriber
        .init();

    // Build a Rocket application
    rocket::build()
        // Configure the port to 8007
        .configure(rocket::Config::figment().merge(("port", 8007)))
        // Mount the OpenAPI specification route at /.well-known
        .mount("/.well-known", routes![open_api_specification])
        // Mount API routes
        .mount(
            "/api",
            routes![
                mint_tweet_request,
                tweet_contract_call,
                tweet_contract_cancel_call
            ],
        )
        // Register error catchers
        .register("/", catchers![unprocessable_entity_catcher, not_found])
}
