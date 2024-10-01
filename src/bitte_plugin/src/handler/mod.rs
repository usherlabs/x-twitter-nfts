use serde::{Deserialize, Serialize};

pub mod catcher_handler;
pub mod open_api_handler;
pub mod tweet;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct PluginInfo {
    pluginId: String,
    url: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct IpfsData {
    IpfsHash: String,
}
