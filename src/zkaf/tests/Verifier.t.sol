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
    bytes correctJournalOutput = hex"87532ef9e4e9d2ae58ce81ed14f5aa9b50babb2dda3a9266af790876c4bc02bd";
    bytes incorrectJournalOutput = hex"5f8ad5ce1d0fc577d04d618880b8c77c9aced63740ca43b708f32425a95b11b7";
    bytes seal = hex"310fe5981b766275439ea2c51bd00cc04d4e8e4071df29561ec6559223a70a124589da5a2d046276699b7c22f2891dffe92a0ed3323770d63973ed0bef406560dfc7efe71f0f2e592e7c95ab6072aa8c4e3701274262f4ba15bd3dbb3652d5b672b8138a09e360d8b9f97f66e0c96ead29ba2e7609214c14351eeb428576345df7990b78281c314e163f3321599af437d75533a36b6562f7380e1433a3c38f90d8c9c31820789673c6cfba165e6cb40ed315533e3c2d21a111ff7fdef834ed7a09b426ea149815dcf692155afe08ea5556c44542b09a4e1d5721963c917dc1c0f32dddf4260330dd0a7ce9cee09e68c54776995ca3ea5de9cb8c4756dddf3b6f81ba3246";

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
