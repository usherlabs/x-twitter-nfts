use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use p256::pkcs8::DecodePublicKey;
use std::str;

#[derive(Serialize, Deserialize)]
pub struct AssetMetadata {
    image_url: String,
    owner_account_id: String,
    token_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ZkInputParam {
    /// verify Proof.
    pub proof: String,

    /// meta_data
    pub meta_data: AssetMetadata,
}

fn build_proof() -> Result<(), Box<dyn std::error::Error>> {
    let proof = std::fs::read_to_string("./fixtures/twitter_proof.json").unwrap();

    let meta_data= AssetMetadata{
        image_url: "https://386f4b0d6749763bc7ab0a648c3e650f.ipfscdn.io/ipfs/QmXPD7KqFyFWwMTQyEo9HuTJjkKLxergS1YTt1wjJNAAHV".to_string(),
        owner_account_id:"xlassixx.testnet".to_string(),
        token_id: "1800368936443379990".to_string(),
    };

    // type conversion occurs here
    // we need to convert from the tlsn core definitions to the definitions from the verifier
    let params = ZkInputParam { proof, meta_data };

    let json = serde_json::to_string(&params)?;

    let file_path = "../inputs/zk_params.json";
    let path = Path::new(file_path);
    // Check if the parent directory exists, and create it if it does not.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Open the file in write mode. This will create the file if it does not exist.
    let mut file = File::create(path)?;
    // Write content to the file.
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Returns a Notary pubkey trusted by this Verifier
pub fn notary_pubkey() -> p256::PublicKey {
    let pem_file = str::from_utf8(include_bytes!("./fixtures/notary.pub")).unwrap();
    p256::PublicKey::from_public_key_pem(pem_file).unwrap()
}

fn main() {
    let _ = build_proof();
}
