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
    bytes32 postStateDigest = 0x296848efeae37d831234c47e5466727fec44c7a74a3388a8a40a2b881f06aa0d;
    bytes seal = hex"17a1316a37214763c58d4d33aeb052d35edaf97053195fc8a7f0aca557a86ea0152901bf0ceec57322bcc524eb9a34b9b295c5a995bbf0ab25262ce3d1dd94b3052121b72d1991457ae636e95630d1586dc01418c05994b771bd2dc46c0c45df12dbd0029963b5e99fb2e055293d8912d77d2be679415c166153245f4f0db7c80120879032b0f47b8d5ecabc2ae51be73ad441bc08fd0c0c4aa7bddd2563c42e2733d05355f730112feeaedaa24b7ebf66deb7013c1facf454b57e144929ec86133ad96a7bf4bc6ed4a187a4bba910bdde6426ce39b1fe7652b03eb0c5b4cfe10f8d5bdac9579f0d0ed7cc3fd5194488c257b3daab08728ef11f4a1c08028ef5";

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
