// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console, VmSafe} from "forge-std/Script.sol";
import {Counter} from "../src/Counter.sol";

contract CounterScript is Script {
    function setUp() public {}

    function run() public {
        VmSafe.Wallet memory wallet = vm.createWallet(
            0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
        );
        vm.startBroadcast(wallet.privateKey);

        Counter counter = Counter(0x610178dA211FEF7D417bC0e6FeD39F05609AD788);

        counter.increment();
        counter.increment();
        uint256 number = counter.number();
        console.logUint(number);
        console.log("Counter incremented");

        vm.stopBroadcast();
    }
}
