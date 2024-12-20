# Auto-Generated Folder: Method

This folder contains generated Rust code for a method used in our zk-SNARK verification process. The contents of this folder should be re-generated based on certain conditions being met and conformed.
NOTE: DO NOT DELETE ANY FILE IN THIS FOLDER

## Conditions for Regeneration

The contents of this folder will only be regenerated under the following circumstances:

1. **Zk Verity Code Change**: Any modification or use of the zk-SNARK verification code has occurred since the last generation. [verity code](../../../zkaf/methods/guest/src/bin/verify.rs)

2. **Contract Deployment**: The contract intended for deployment on Aurora has been successfully deployed. if Not deployed see deployment [Readme](../../../zkaf/deployment-guide.md)

3. **Aurora Contract Update Validation**: It has been validated that the Aurora contract address specified in the verifier contract has been updated to reflect the newly deployed contract.
 - To check verifier contract data [see](../../../contracts/verifier/contract/scripts/get_contract_data.sh)
 - To update aurora contract address on the verifier contract [see](../../../contracts/verifier/contract/scripts/set_verifier.sh)

## Contents

The folder contains a single generate Rust file:

- `method.rs`: This file contains the generated method implementation for our zk-SNARK verification process.


## Usage

To ensure you're using the most up-to-date version of this method:

1. Verify that all conditions for regeneration have been met.
2. Run the auto-generation script (not included in this folder).
3. Update your project dependencies if necessary.

## Important Notes

- Always review the generated code before integrating it into your project.
- Ensure that your development environment is properly set up with all required dependencies.
- If you encounter any issues or discrepancies, please consult with the team responsible for the zk-SNARK implementation.

## To Copy ELF
```
cd src/zkaf && cargo run --bin copy_elf
```
