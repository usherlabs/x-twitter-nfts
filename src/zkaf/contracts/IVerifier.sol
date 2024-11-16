// Copyright (c) 2024 RISC Zero, Inc.
//
// All rights reserved.

pragma solidity ^0.8.20;

interface IVerifier {
    function isJournalVerified(bytes calldata journalData) external view returns (bool);
    function verify_proof(bytes memory journalOutput, bytes calldata seal) external;
}
