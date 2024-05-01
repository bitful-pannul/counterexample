use kinode_process_lib::{await_message, call_init, eth::{Address as EthAddress, Provider, TransactionInput, TransactionRequest, U256}, println, Address, Response};
use alloy_sol_types::{sol, SolCall, SolValue};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

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

pub const COUNTER_ADDRESS: &str = "0x1234567890123456789012345678901234567890";

#[derive(Debug, Deserialize, Serialize)]
pub enum CounterAction {
    Increment,
    Read,
}

fn handle_message(_our: &Address, provider: &Provider) -> anyhow::Result<()> {
    let message = await_message()?;

    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let action: CounterAction = serde_json::from_slice(message.body())?;

    match action {
        CounterAction::Increment => {
            println!("this next!");
        }
        CounterAction::Read => {
            let _count = read(&provider);
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
        .to(Some(counter_address))
        .input(TransactionInput::new(count.into()));
    let x = provider.call(tx, None);

    match x {
        Ok(b) => {
            let number = U256::abi_decode(&b, false).unwrap();
            println!("current count: {:?}", number);
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

    loop {
        match handle_message(&our, &provider) {
            Ok(()) => {}
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}
