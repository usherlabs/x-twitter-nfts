// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

interface IVerifier {
    /// @notice RISC Zero verifier contract address.
    function getVerifier() external view returns (address);

    /// @notice Image ID of the only zkVM binary to accept verification from.
    // bytes32 public constant IMAGE_ID;

    /// @notice mapping to keep track of if a journal is verified
    function isJournalVerified(bytes calldata) external view returns (bool);

    /// @notice Emitted when a proof is verified.
    event ProofVerified(address indexed user, bytes journal);
}
