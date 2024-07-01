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

#[cfg(test)]
mod tests {
    use alloy_sol_types::SolValue;
    use risc0_zkvm::{default_executor, ExecutorEnv};
    use tlsn_substrings_verifier::ZkInputParam;

    #[test]
    fn proves_verification() {
        let proof_params = std::fs::read_to_string("fixtures/zk_params.json").unwrap();
        let proof_params: ZkInputParam = serde_json::from_str(proof_params.as_str()).unwrap();
        let expected_req_res_hash = "87532ef9e4e9d2ae58ce81ed14f5aa9b50babb2dda3a9266af790876c4bc02bd".to_string();

        let input = serde_json::to_string(&proof_params).unwrap();
        let input: &[u8] = input.as_bytes();

        let env = ExecutorEnv::builder()
            .write_slice(input)
            .build()
            .unwrap();

        // NOTE: Use the executor to run tests without proving.
        let session_info = default_executor().execute(env, super::VERIFY_ELF).unwrap();

        let req_res_hash = <Vec<u8>>::abi_decode(&session_info.journal.bytes, true).unwrap();
        let req_res_hash_hex_string = hex::encode(&req_res_hash);

        assert_eq!(req_res_hash_hex_string, expected_req_res_hash);
    }
}
