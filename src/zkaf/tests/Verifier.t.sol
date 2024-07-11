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

import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {Verifier} from "../contracts/Verifier.sol";
import {Elf} from "./Elf.sol"; // auto-generated contract after running `cargo build`.

contract VerifierTest is RiscZeroCheats, Test {
    Verifier public proof;
    bytes correctJournalOutput = hex"94e9b167bbc12315ff0ee6cfaa6e128d889ae6a25035ccacb7fff8a4c7609594";
    bytes incorrectJournalOutput = hex"5f8ad5ce1d0fc577d04d618880b8c77c9aced63740ca43b708f32425a95b11b7";
    bytes seal = hex"310fe5982e98d2c57198e03159c1537af7d1568241a8fd95b4395a907067cda3b4ff15d30600b768767771024cc024352f42bac841f7cb91cd8d8591260a9430230f23491bc18634b1f118eb56c955cf04442f485198855d9c2ed4b1e5db11e5d5356fc021d0e949522de07cbbce350c3ce8475b2d669ff1aa8c839c01e710b1b8a38fdc23ad11a620c2fb72c1ba434050658a2aa00c4a8726f78fb2ed2eb811eb3d15932f6176c5b0b2bdc104e659a5b09c2755b56f2711b07065ad46550ad874b8fb791b493721f11a6306f9ddd8bd6ac8a2048438cda06924866e708b4f9dc1dfe1770d76b09b3788b595bfedfcc47720c5662661a0575c3c4956e1cbe4d22a99708c";

    function setUp() public {
        IRiscZeroVerifier verifier = deployRiscZeroVerifier();
        proof = new Verifier(verifier);
    }

    function testSnarkVerificationSuccess() public {

        proof.verify_proof(
            correctJournalOutput,
            seal
        );

        assert(proof.isJournalVerified(correctJournalOutput));
    }

    function testSnarkVerificationFailure() public {
        vm.expectRevert();
        proof.verify_proof(
            incorrectJournalOutput,
            seal
        );
    }
}
