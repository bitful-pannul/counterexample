use alloy_consensus::{SignableTransaction, TxEnvelope, TxLegacy};
use alloy_network::TxSignerSync;
use alloy_primitives::TxKind;
use alloy_rlp::Encodable;
use alloy_rpc_types::TransactionRequest;
use alloy_signer_wallet::LocalWallet;
use alloy_sol_types::{sol, SolCall, SolValue};
use kinode_process_lib::{
    await_message, call_init,
    eth::{Address as EthAddress, Provider, TransactionInput, U256},
    println, Address, Response,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
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

pub const COUNTER_ADDRESS: &str = "0x610178dA211FEF7D417bC0e6FeD39F05609AD788";

#[derive(Debug, Deserialize, Serialize)]
pub enum CounterAction {
    Increment,
    Read,
    SetNumber(u64),
}

fn handle_message(_our: &Address, provider: &Provider, wallet: &LocalWallet) -> anyhow::Result<()> {
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
            let mut tx = TxLegacy {
                chain_id: Some(31337),
                nonce: nonce,
                to: TxKind::Call(EthAddress::from_str(COUNTER_ADDRESS).unwrap()),
                gas_limit: 100000,
                gas_price: 100000000,
                input: increment.into(),
                ..Default::default()
            };

            let sig = wallet.sign_transaction_sync(&mut tx)?;
            let signed = TxEnvelope::from(tx.into_signed(sig));

            let mut buf = vec![];
            signed.encode(&mut buf);

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
            let mut tx = TxLegacy {
                chain_id: Some(31337),
                nonce: nonce,
                to: TxKind::Call(EthAddress::from_str(COUNTER_ADDRESS).unwrap()),
                gas_limit: 100000,
                gas_price: 100000000,
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
        .input(TransactionInput::new(count.into()));
    let x = provider.call(tx, None);

    match x {
        Ok(b) => {
            let number = U256::abi_decode(&b, false).unwrap();
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

    let wallet =
        LocalWallet::from_str("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
            .unwrap();

    loop {
        match handle_message(&our, &provider, &wallet) {
            Ok(()) => {}
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}
