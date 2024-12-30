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

/// Parses constants from the input string and returns the ELF file array as a formatted string.
///
/// This function looks for a line containing "VERIFY_ELF" in the input string, extracts the ELF bytes,
/// cleans them up, and formats them as a constant declaration.
///
/// # Arguments
///
/// * `input` - The input string containing the source code to parse.
///
/// # Returns
///
/// * `Ok(String)` - The formatted string declaring the ELF file array constant.
/// * `Err(Box<dyn std::error::Error>)` - If an error occurs during parsing or reading.
fn parse_constants(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = input.split('\n').collect();

    // Collect lines starting with "VERIFY_ELF"
    let selected_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.contains("VERIFY_ELF"))
        .map(|line| line.to_string())
        .collect();

    // Check if any matching lines were found
    if selected_lines.is_empty() {
        return Err("Could not find the line".into());
    }

    // Extract the ELF bytes from the selected line
    let elf_bytes: String = selected_lines[0].split("include_bytes!").skip(1).collect();

    // Clean up the extracted string by removing unnecessary characters
    let cleaned_elf_bytes = elf_bytes
        .replace("(", "")
        .replace("\"", "")
        .replace(")", "")
        .replace(";", "")
        .replace(" ", "");

    // Read the actual bytes from the cleaned string
    let byte = fs::read(cleaned_elf_bytes).unwrap();

    // Format the result as a constant declaration
    Ok(format!("pub const VERIFY_ELF: &[u8] = &{:?};", byte))
}

/// Copies the ELF file from the build output to the specified destination folder.
///
/// # Parameters
///
/// - `dest_folder`: The name of the folder where the ELF file should be copied.
///
/// # Returns
///
/// - `Ok(())`: If the operation was successful.
/// - `Err(Box<dyn std::error::Error>)`: If an error occurred during the process.
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

    // Check if the content contains "include_bytes!" (used for embedded resources)
    if content.contains("include_bytes!") {
        // If it's an embedded resource, write the parsed constants
        fs::write(&dest_path, &parse_constants(&content).unwrap())?;
    } else {
        // Otherwise, write the original content
        fs::write(&dest_path, &content)?;
    }

    println!("File copied successfully to: {}", dest_path.display());

    Ok(())
}
#[cfg(test)]
mod tests {
    use alloy_sol_types::SolValue;
    use risc0_zkvm::{default_executor, ExecutorEnv};

    use crate::{parse_constants, VERIFY_ELF};

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
