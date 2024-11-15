pub mod contract_utils;
pub mod near_interface;

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::{
        contract_utils::{remove_quotes, Verifier, VerifierTestContext},
        near_interface,
    };
    use aurora_sdk_integration_tests::{
        aurora_engine,
        aurora_engine_types::{parameters::engine::TransactionStatus, types::Wei},
        tokio, utils, workspaces,
    };

    #[tokio::test]
    async fn test_contract() {
        let dummy_evm_address = "0x99d7584971A1E0Fb6409108c5106323b2578aeeE".to_lowercase();
        let journal_output: Vec<u8> =
            hex::decode("94e9b167bbc12315ff0ee6cfaa6e128d889ae6a25035ccacb7fff8a4c7609594")
                .unwrap();
        let seal: Vec<u8> = hex::decode("310fe5982e98d2c57198e03159c1537af7d1568241a8fd95b4395a907067cda3b4ff15d30600b768767771024cc024352f42bac841f7cb91cd8d8591260a9430230f23491bc18634b1f118eb56c955cf04442f485198855d9c2ed4b1e5db11e5d5356fc021d0e949522de07cbbce350c3ce8475b2d669ff1aa8c839c01e710b1b8a38fdc23ad11a620c2fb72c1ba434050658a2aa00c4a8726f78fb2ed2eb811eb3d15932f6176c5b0b2bdc104e659a5b09c2755b56f2711b07065ad46550ad874b8fb791b493721f11a6306f9ddd8bd6ac8a2048438cda06924866e708b4f9dc1dfe1770d76b09b3788b595bfedfcc47720c5662661a0575c3c4956e1cbe4d22a99708c").unwrap();

        let worker = workspaces::sandbox().await.unwrap();
        let engine = aurora_engine::deploy_latest(&worker).await.unwrap();

        // deploy the verifier near contract
        let verifier_contract_bytes = utils::cargo::build_contract("../contract").await.unwrap();
        let near_verifier_contract_proxy = near_interface::VerifierProxy {
            contract: worker.dev_deploy(&verifier_contract_bytes).await.unwrap(),
        };

        // deploy the nft near contract
        let nft_contract_wasm = std::fs::read("./wasm/non_fungible_token.wasm").unwrap();
        let near_nft_contract_proxy = near_interface::NFTContractProxy {
            contract: worker.dev_deploy(&nft_contract_wasm).await.unwrap(),
        };

        // Deploy evm contracts and get verifier contract
        let evm_context = VerifierTestContext::new(
            engine.clone(),
            near_verifier_contract_proxy.contract.as_account().clone(),
        )
        .await;

        // Obtain the important addresses and accounts
        let evm_verifier_address = format!("0x{}", evm_context.verifier.0.address.encode());
        let near_verifier_account_id = near_verifier_contract_proxy.contract.as_account().id();
        let near_nft_account_id = near_nft_contract_proxy.contract.as_account().id();

        println!("near nft_account_id:{}", near_nft_account_id);
        println!("verifier_address:{}", evm_verifier_address);
        println!("near verifier_account_id: {}", near_verifier_account_id);

        // initialize both the verifier and NFT contract
        near_verifier_contract_proxy
            .initialize(
                &engine.inner.id(),
                &evm_verifier_address,
                near_nft_account_id,
            )
            .await
            .unwrap();
        near_nft_contract_proxy
            .initialize(near_verifier_account_id)
            .await
            .unwrap();

        // Change the verifier and test the value truly changed
        near_verifier_contract_proxy
            .set_verifier_address(&dummy_evm_address)
            .await
            .unwrap();
        let result = near_verifier_contract_proxy
            .get_verifier_address()
            .await
            .unwrap();
        let result = remove_quotes(&result);
        assert_eq!(result, dummy_evm_address);

        // set the verifier contract to the original one so further tests may access the right verifier contract
        near_verifier_contract_proxy
            .set_verifier_address(&evm_verifier_address)
            .await
            .unwrap();

        // perform the verification on aurora with testing parameters
        let input = evm_context
            .verifier
            .create_verify_proof_bytes(journal_output.clone(), seal);
        let mint_result = engine
            .call_evm_contract(evm_context.verifier_contract_address, input, Wei::zero())
            .await
            .unwrap();
        if let TransactionStatus::Revert(_) = mint_result.status {
            panic!("EVM verification transaction reverted");
        }

        // perform the verification/minting on near
        let test_token_metadata = Verifier::generate_default_metadata();
        near_verifier_contract_proxy
            .verify_proof(journal_output, test_token_metadata.clone())
            .await
            .unwrap();

        // fetch the nft and verify it exists
        let newly_minted_token = near_nft_contract_proxy
            .nft_token_by_id(test_token_metadata.clone().title.unwrap())
            .await
            .unwrap()
            .expect("NFT not minted");

        // validate the token details
        assert_eq!(
            newly_minted_token.clone().metadata.unwrap().title.unwrap(),
            test_token_metadata.title.unwrap(),
            "NFT title does not match"
        );
        assert_eq!(
            newly_minted_token
                .clone()
                .metadata
                .unwrap()
                .description
                .unwrap(),
            test_token_metadata.description.unwrap(),
            "NFT description does not match"
        );

        // validate the token owner
        assert_eq!(
            newly_minted_token.clone().owner_id.to_string(),
            near_verifier_account_id.clone().to_string()
        )
    }
}
