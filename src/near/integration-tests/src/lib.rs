#[cfg(test)]
mod tests {
    use aurora_sdk_integration_tests::{
        tokio, utils, workspaces,
    };

    fn remove_quotes(input: &str) -> String {
        input.chars()
            .filter(|&c| c != '\'' && c != '\"')
            .collect()
    }


    #[tokio::test]
    async fn test_contract() {
        let dummy_address = "0x89d7584971A1E0Fb6409108c5106323b2578aeeE".to_lowercase();

        let worker = workspaces::sandbox().await.unwrap();

        // This is needed because of a quirk of how `cargo-near` works. It doesn't handle
        // cargo workspaces properly yet.
        tokio::fs::create_dir_all("../target/near/uniswap_from_near")
            .await
            .unwrap();
        let contract_bytes = utils::cargo::build_contract("../contract").await.unwrap();
        let contract = contract_interface::VerifierProxy {
            contract: worker.dev_deploy(&contract_bytes).await.unwrap(),
        };

        // Initialize our VerifierProxy contract
        contract
            .create(&dummy_address)
            .await
            .unwrap();

        // Get the verifier address and verify its the same
        let result = contract.get_verifier_address().await.unwrap();
        let result = remove_quotes(&result);
        assert_eq!(result, dummy_address);
        
        // Change the verifier and test the value
        let dummy_address = "0x99d7584971A1E0Fb6409108c5106323b2578aeeE".to_lowercase();
        contract.set_verifier_address(&dummy_address).await.unwrap();
        let result = contract.get_verifier_address().await.unwrap();
        let result = remove_quotes(&result);
        assert_eq!(result, dummy_address);

        // assert_eq!(result, dummy_address.to_string())
    }


    mod contract_interface{
        use aurora_sdk_integration_tests::workspaces::{self, Contract};

        pub struct VerifierProxy{
            /// The `workspaces::Contract` instance here must have the UniswapProxy example
            /// contract deployed; it cannot be any `Contract`.
            pub contract: Contract
        }

        impl VerifierProxy{
            pub async fn create(
                &self,
                contract_address: &str,
            ) -> Result<(), workspaces::error::Error> {
                let result = self
                    .contract
                    .call("init")
                    .args_json(NewArgs {
                        contract_address,
                    })
                    .max_gas()
                    .transact()
                    .await?;
                result.into_result()?;
                Ok(())
            }

            pub async fn get_verifier_address(&self) -> Result<String, workspaces::error::Error>  {

                let response = self.contract.call("get_verifier_address").max_gas()
                .view()
                .await?;

                let result = response.result;
                let result = String::from_utf8(result).unwrap(); 
                
                return Ok(result);
            }

            pub async fn set_verifier_address(
                &self,
                new_contract_address: &str,
            ) -> Result<(), workspaces::error::Error> {
                let result = self
                    .contract
                    .call("set_verifier_address")
                    .args_json(SetArgs {
                        new_contract_address,
                    })
                    .max_gas()
                    .transact()
                    .await?;
                result.into_result()?;
                Ok(())
            }
               
        }

        #[derive(serde::Serialize)]
        struct NewArgs<'a> {
            contract_address: &'a str
        }


        #[derive(serde::Serialize)]
        struct SetArgs<'a> {
            new_contract_address: &'a str
        }

    }
}