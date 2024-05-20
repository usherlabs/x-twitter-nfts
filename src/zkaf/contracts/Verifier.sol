// Copyright 2024 RISC Zero, Inc.
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
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {ImageID} from "./ImageID.sol"; // auto-generated contract after running `cargo build`.

/// @title An application using RISC Zero.
/// @notice This basic application holds a number, guaranteed to be even.
/// @dev This contract demonstrates one pattern for offloading the computation of an expensive
///      or difficult to implement function to a RISC Zero guest running on Bonsai.
contract Verifier {
    /// @notice RISC Zero verifier contract address.
    IRiscZeroVerifier public immutable verifier;
    /// @notice Image ID of the only zkVM binary to accept verification from.
    bytes32 public constant imageId = ImageID.VERIFY_ID;

    /// @notice A number that is guaranteed, by the RISC Zero zkVM, to be even.
    ///         It can be set by calling the `set` function.
    uint256 public number;

    event ProofVerified(address indexed user, bytes journal);

    /// @notice Initialize the contract, binding it to a specified RISC Zero verifier.
    constructor(IRiscZeroVerifier _verifier) {
        verifier = _verifier;
        number = 0;
    }

    /// @notice verifies a proof.
    // function verify_proof(bytes memory journal_output, bytes32 postStateDigest, bytes calldata seal) public view returns(uint256) {
    function verify_proof() public view returns(uint256) {
        // converts this function to return verification output 
        bytes memory journal_output = hex"4f8ad5ce1d0fc577d04d618880b8c77c9aced63740ca43b708f32425a95b11b7";
        bytes32 postStateDigest = 0x0622b43513c8148548a5aa01d3842e669df2ed7cae1b498e98ae1a8b3aae1a14;
        bytes memory seal = hex"1a61e905022e6727f77f55f5632d1b418040e236961222301b1b0df883d8124c225b443b81443529841cbd6845e65efc7a36547ff66003d5e94ac36f91ca83f42c81d508177c6b03d647ccc662e83ba94f0814f941dbef13dc374eea5897bd5a19578107ca5766c68e0a49b4576c0e3d30d4c82f5bdd34ffff81673d2141b6d6163b8ac07ddd56827824da2196046baade5cfd64dd3d0046a4faa1c0b2602f762a2d547b5bf7aaadca7c1f69f5a97eebf7f9c810fe420ab846081f7a8a16e1172735b48780dc334aaa3a8c4806c02a396746e6500cee439f8bf9b6de1c43a0e81345aac7a3fce0e8eed676b4307611b7b8e567f1e6f14333e29e032843a0fe1b";
        
        bytes memory journal = abi.encode(journal_output);
        // Construct the expected journal data. Verify will fail if journal does not match.
        bool isVerified = verifier.verify(seal, imageId, postStateDigest, sha256(journal));
        return isVerified? 1: 0;
    }

    /// @notice Returns the number stored.
    function get() public view returns (uint256) {
        return number;
    }
}
