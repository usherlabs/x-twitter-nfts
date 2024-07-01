# Near Contract

The near contract is responsible for checking if a proof has been verified on the aurora chain, it does this my checking a mapping `isJournalVerified` on the aurora smart contract and throws an error if the variable returns false, indicating it has not been verified.

### Prerequisites:

- [ ]  A near testnet account (https://testnet.mynearwallet.com/)
- [ ]  Near Rust CLI (https://docs.near.org/tools/near-cli-rs)
- [ ]  Ensure `wasm32-unknown-unknown` is installed
       ```shell
        rustup target add wasm32-unknown-unknown
       ```
- [ ]  Login to near on the CLI by running  `sh login.sh`

### Deployment

- Initial deployment can be performed by running `sh deploy_contract.sh`
- subsequently after initial deployment, further deployments are considered upgrades to the contract and can be persisted by running `sh upgrade_contract.sh`

### Calling the contract

The respective methods on the contract can be called by running the corresponding script in the `scripts` directory.

### Testing the contract

The contract can be tested by running `cargo test` at the root of the `integration-tests` folder.

### Workspace organization

This example is organized into two crates: the contract that would be deployed to a Near network, and a crate for integration testing of the contract.
This layout was chosen as opposed to making the integration tests part of a [tests directory](https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory) in the contract crate because the purpose of the integration tests is to test the compiled Wasm (i.e. binary) artifact of the contract as opposed to testing it as a Rust library.
My understanding of the [note in the Rust book about the tests directory](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests-for-binary-crates) is that it is meant for testing library integrations as opposed to binary integrations.
Thus I chose to factor the integration tests out as an entirely separate crate.
You may or may not make a different choice in your own projects, and that's ok.
