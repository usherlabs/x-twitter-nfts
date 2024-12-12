// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Generated crate containing the image ID and ELF binary of the build guest.
include!(concat!(env!("OUT_DIR"), "/methods.rs"));

use std::fs;
use std::path::PathBuf;

pub fn copy_elf(dest_folder: String) -> Result<(), Box<dyn std::error::Error>> {
    // Construct the source path
    let src_path = PathBuf::from(env!("OUT_DIR")).join("methods.rs");

    // Read the contents of the source file
    let content = fs::read_to_string(&src_path)?;

    let dest_path = PathBuf::from(dest_folder);

    // Create the destination folder if it doesn't exist
    fs::create_dir_all(&dest_path)?;

    // Construct the destination file path
    let dest_file = src_path.file_name().unwrap().to_os_string();
    let dest_path = dest_path.join(dest_file);

    // Write the content to the destination file
    fs::write(&dest_path, &content)?;

    println!("File copied successfully to: {}", dest_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::SolValue;
    use risc0_zkvm::{default_executor, ExecutorEnv};

    #[test]
    fn proves_verification() {
        let proof_params = std::fs::read_to_string("../fixtures/zk_params.json").unwrap();

        let input: &[u8] = proof_params.as_bytes();

        let env = ExecutorEnv::builder().write_slice(input).build().unwrap();

        // NOTE: Use the executor to run tests without proving.
        let session_info = default_executor().execute(env, super::VERIFY_ELF).unwrap();

        let req_res_hash = <Vec<u8>>::abi_decode(&session_info.journal.bytes, true).unwrap();
        let req_res_hash_hex_string = hex::encode(&req_res_hash);

        assert!(req_res_hash_hex_string.len() > 10);
    }
}
