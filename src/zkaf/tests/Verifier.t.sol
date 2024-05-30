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

import {RiscZeroCheats} from "risc0/RiscZeroCheats.sol";
import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {Verifier} from "../contracts/Verifier.sol";
import {Elf} from "./Elf.sol"; // auto-generated contract after running `cargo build`.

contract VerifierTest is RiscZeroCheats, Test {
    Verifier public proof;
    bytes correctJournalOutput = hex"4f8ad5ce1d0fc577d04d618880b8c77c9aced63740ca43b708f32425a95b11b7";
    bytes incorrectJournalOutput = hex"5f8ad5ce1d0fc577d04d618880b8c77c9aced63740ca43b708f32425a95b11b7";
    bytes32 postStateDigest = 0x861ffac3fc1be3ec222139c10070c668284c85b2e8906cc2546b889c6ca8fe2d;
    bytes seal = hex"028ddfd058a2e818bf25fb9d710359042fb5e7200c28105944b2b9c8c3e2af4d1cb77b9730fcb22b0c182c791678d6895ac436cab7fb99e15e773d2a97e4e1d526ee8e31b492c6ebfc66418ffa8c0e5ed0b84ca66d5a35174640bc01c8f779a218d3be19a74b952697f7fe73520e2d12795309c67e9e255d5a890bb7aa415bbe277d73cc765337c91c2f65c8f4617144bc4bc76bf95f01d49328e27de3820a6910a2bdc481bb93ddaf6d688223abad6a53245be7676e505e702ff0b4cb5d5d6a22c30923e5db696f5c94dc9a3ad736d909eec7da10ce00476b31f5d678ae05dc1e293717c3dafe22a942f693ebfb4c75eb5c08f75dd1e45b74d9f455860088b2";

    function setUp() public {
        IRiscZeroVerifier verifier = deployRiscZeroVerifier();
        proof = new Verifier(verifier);
    }

    function testSnarkVerificationSuccess() public {

        proof.verify_proof(
            correctJournalOutput,
            postStateDigest,
            seal
        );

        assert(proof.isJournalVerified(correctJournalOutput));
    }

    function testSnarkVerificationFailure() public {
        vm.expectRevert();
        proof.verify_proof(
            incorrectJournalOutput,
            postStateDigest,
            seal
        );
    }

}
