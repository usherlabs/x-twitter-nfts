use rocket::serde::json::{self};
use std::{env, fs, path::PathBuf};

use super::PluginInfo;

pub fn extract_plugin_url() -> String {
    env::var("HOST_URL").unwrap_or_else(|_| {
        let current_dir = env::current_dir().unwrap();
        let mut bitte_config_path = PathBuf::from(current_dir);
        bitte_config_path.push(".env");
        let bitte_config = fs::read_to_string(bitte_config_path).unwrap();
        // Split the contents into lines
        let lines: Vec<&str> = bitte_config.split('\n').collect();

        // Collect lines starting with "BITTE_CONFIG"
        let config_lines: Vec<String> = lines
            .iter()
            .filter(|line| line.starts_with("BITTE_CONFIG"))
            .map(|line| line.to_string())
            .collect();
        if config_lines.len() == 0 {
            return "".to_string();
        }
        let plugin_info: PluginInfo = json::serde_json::from_str(
            config_lines
                .first()
                .unwrap()
                .replace("BITTE_CONFIG=", "")
                .as_str(),
        )
        .unwrap();
        plugin_info.url
    })
}
