
use aurora_sdk_integration_tests::workspaces::{
    self,
    result::Value,
    Contract,
};
use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, Token, TokenId};

pub struct VerifierProxy {
    /// The `workspaces::Contract` instance here must have the
    /// contract deployed; it cannot be any `Contract`.
    pub contract: Contract,
}

impl VerifierProxy {
    pub async fn initialize(
        &self,
        aurora: &workspaces::AccountId,
        contract_address: &str,
        nft_account_id: &workspaces::AccountId,
    ) -> Result<(), workspaces::error::Error> {
        let result = self
            .contract
            .call("init")
            .args_json(NewVerifierArgs {
                aurora: aurora.clone(),
                contract_address,
                nft_account_id: nft_account_id.clone(),
            })
            .max_gas()
            .transact()
            .await?;
        result.into_result()?;
        Ok(())
    }

    pub async fn get_verifier_address(&self) -> Result<String, workspaces::error::Error> {
        let response = self
            .contract
            .call("get_verifier_address")
            .max_gas()
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

    pub async fn verify_proof(
        &self,
        journal: Vec<u8>,
        token_metadata: TokenMetadata,
    ) -> Result<workspaces::result::ExecutionResult<Value>, workspaces::error::Error> {
        let result = self
            .contract
            .call("verify_proof")
            .args_json(VerifyProofArgs {
                journal,
                token_metadata,
            })
            .max_gas()
            .transact()
            .await?;
        let res = result.into_result().unwrap();
        Ok(res)
    }
}

pub struct NFTContractProxy {
    /// The `workspaces::Contract` instance here must have the
    /// contract deployed; it cannot be any `Contract`.
    pub contract: Contract,
}

impl NFTContractProxy {
    pub async fn initialize(
        &self,
        owner_id: &workspaces::AccountId,
    ) -> Result<(), workspaces::error::Error> {
        let result = self
            .contract
            .call("new_default_meta")
            .args_json(NewNFTArgs {
                owner_id: owner_id.clone(),
            })
            .max_gas()
            .transact()
            .await?;
        result.into_result()?;
        Ok(())
    }

    pub async fn nft_token_by_id(
        &self,
        token_id: TokenId,
    ) -> Result<Option<Token>, workspaces::error::Error> {
        let token: Option<Token> = self
            .contract
            .call("nft_token")
            .args_json(QueryNFTArgs {
                token_id: token_id.clone(),
            })
            .max_gas()
            .transact()
            .await?
            .json()?;
        Ok(token)
    }
}
#[derive(serde::Serialize)]
pub struct NewVerifierArgs<'a> {
    pub aurora: workspaces::AccountId,
    pub contract_address: &'a str,
    pub nft_account_id: workspaces::AccountId,
}

#[derive(serde::Serialize)]
pub struct NewNFTArgs {
    pub owner_id: workspaces::AccountId,
}

#[derive(serde::Serialize)]
pub struct QueryNFTArgs {
    pub token_id: TokenId,
}

#[derive(serde::Serialize)]
pub struct SetArgs<'a> {
    pub new_contract_address: &'a str,
}

#[derive(serde::Serialize)]
pub struct VerifyProofArgs {
    pub journal: Vec<u8>,
    pub token_metadata: TokenMetadata,
}
