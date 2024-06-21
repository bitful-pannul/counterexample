use alloy::{
    consensus::{SignableTransaction, TxEip1559, TxEnvelope},
    network::{eip2718::Encodable2718, TxSignerSync},
    primitives::TxKind,
    rpc::types::eth::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use alloy_rlp::Encodable;
use alloy_sol_types::{sol, SolCall, SolValue};
use kinode_process_lib::{
    await_message, call_init,
    eth::{Address as EthAddress, Provider, U256},
    println, Address, Response,
};

use serde::{Deserialize, Serialize};
use std::str::FromStr;

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

sol! {
    contract Counter {
        uint256 public number;

        function setNumber(uint256 newNumber) public {
            number = newNumber;
        }

        function increment() public {
            number++;
        }
    }
}

pub const COUNTER_ADDRESS: &str = "0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82";

#[derive(Debug, Deserialize, Serialize)]
pub enum CounterAction {
    Increment,
    Read,
    SetNumber(u64),
}

fn handle_message(
    _our: &Address,
    provider: &Provider,
    wallet: &PrivateKeySigner,
) -> anyhow::Result<()> {
    let message = await_message()?;

    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let action: CounterAction = serde_json::from_slice(message.body())?;

    match action {
        CounterAction::Read => {
            let _count = read(&provider);
        }
        CounterAction::Increment => {
            let increment = Counter::incrementCall {}.abi_encode();

            let nonce = provider
                .get_transaction_count(wallet.address(), None)
                .unwrap()
                .to::<u64>();

            let gas = provider
                .get_gas_price()
                .map_err(|e| anyhow::anyhow!("couldn't get gas price: {:?}", e))?;

            // todo: use TransactionRequest once signature encoding works.
            let mut tx = TxEip1559 {
                chain_id: 31337,
                nonce: nonce,
                to: TxKind::Call(EthAddress::from_str(COUNTER_ADDRESS).unwrap()),
                gas_limit: 15000000,
                max_fee_per_gas: 10000000000,
                max_priority_fee_per_gas: 300000000,
                input: increment.into(),
                ..Default::default()
            };
            // let gas = provider.estimate_gas(tx.clone().into(), None);
            let sig = wallet.sign_transaction_sync(&mut tx)?;

            let signed = TxEnvelope::from(tx.into_signed(sig));

            let mut buf = vec![];
            signed.encode_2718(&mut buf);

            let tx_hash = provider.send_raw_transaction(buf.into());
            println!("tx_hash: {:?}", tx_hash);
        }
        CounterAction::SetNumber(n) => {
            let set_number = Counter::setNumberCall {
                newNumber: U256::from(n),
            }
            .abi_encode();

            let nonce = provider
                .get_transaction_count(wallet.address(), None)
                .unwrap()
                .to::<u64>();

            let mut tx = TxEip1559 {
                chain_id: 31337,
                nonce: nonce,
                to: TxKind::Call(EthAddress::from_str(COUNTER_ADDRESS).unwrap()),
                gas_limit: 15000000,
                max_fee_per_gas: 10000000000,
                max_priority_fee_per_gas: 300000000,
                input: set_number.into(),
                ..Default::default()
            };
            let sig = wallet.sign_transaction_sync(&mut tx)?;
            let signed = TxEnvelope::from(tx.into_signed(sig));

            let mut buf = vec![];
            signed.encode(&mut buf);

            let tx_hash = provider.send_raw_transaction(buf.into());
            println!("tx_hash: {:?}", tx_hash);
        }
    }

    Response::new()
        .body(serde_json::to_vec(&serde_json::json!("Ack")).unwrap())
        .send()
        .unwrap();
    Ok(())
}

fn read(provider: &Provider) -> anyhow::Result<U256> {
    let counter_address = EthAddress::from_str(COUNTER_ADDRESS).unwrap();
    let count = Counter::numberCall {}.abi_encode();

    let tx = TransactionRequest::default()
        .to(counter_address)
        .input(count.into());

    let x = provider.call(tx, None);

    match x {
        Ok(b) => {
            let number = U256::abi_decode(&b, false)?;
            println!("current count: {:?}", number.to::<u64>());
            Ok(number)
        }
        Err(e) => {
            println!("error getting current count: {:?}", e);
            Err(anyhow::anyhow!("error getting current count: {:?}", e))
        }
    }
}

call_init!(init);
fn init(our: Address) {
    println!("begin");

    let provider = Provider::new(31337, 5);

    let _count = read(&provider);

    let wallet = PrivateKeySigner::from_str(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    )
    .unwrap();

    // EthereumWallet doesn't have proper traits yet.
    // let wallet: EthereumWallet = wallet.into();

    loop {
        match handle_message(&our, &provider, &wallet) {
            Ok(()) => {}
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}
