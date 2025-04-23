use reqwest::{
    Client,
    multipart::{Form,Part}};
use rocket::serde::json;
use std::{env, fs, path::PathBuf};

use super::{IpfsData, LighthouseRespone, PluginInfo};

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


pub async fn lighthouse_upload(image: Vec<u8>) -> Result<String,String> {

    let client = Client::new();

    let form = Form::new().part("file", Part::bytes(image).file_name("image.png"));

    let lighthouse_token = env::var("LIGHTHOUSE_TOKEN").expect("LIGHTHOUSE_TOKEN must be set");

    let url = "https://node.lighthouse.storage/api/v0/add?pin=true";
    let response = client
        .post(url)
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .header("Encryption", "false")
        .header("Mime-Type", "null")
        .header("Authorization", format!("Bearer {}", lighthouse_token))
        .multipart(form)
        .send()
        .await;

    if response.is_err() {
        return Err(format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed")));
    }

    let response = response.unwrap().json::<LighthouseRespone>().await;

    if response.is_err() {
        return Err(format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed")));
    }

    let image_url = format!(
        "https://gateway.lighthouse.storage/ipfs/{}",
        response.unwrap().Hash
    );
    return  Ok(image_url);
}


pub async fn pinata_upload(image: Vec<u8>) -> Result<String,String> {
    let pinata_token = env::var("PINATA_TOKEN").expect("PINATA_TOKEN must be set");

    let client = Client::new();

    let form = Form::new()
        .part("file", Part::bytes(image).file_name("image.png"))
        .part("pinataOptions", Part::text("{\"wrapWithDirectory\":false}"))
        .part(
            "pinataMetadata",
            Part::text("{\"name\":\"Storage SDK\",\"keyvalues\":{}}"),
        );

    let url = "https://api.pinata.cloud/pinning/pinFileToIPFS";
    let response = client
        .post(url)
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .header("Authorization", format!("Bearer {}", pinata_token))
        .multipart(form)
        .send()
        .await;

    if response.is_err() {
        return Err(format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed")));
    }

    let response = response.unwrap().json::<IpfsData>().await;

    if response.is_err() {
        return Err(format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed")));
    }

    let image_url = format!(
        "https://violet-charming-canidae-581.mypinata.cloud/ipfs/{}",
        response.unwrap().IpfsHash
    );
    return  Ok(image_url);
}
