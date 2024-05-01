use kinode_process_lib::{await_message, call_init, eth::{Address as EthAddress, Provider, TransactionInput, TransactionRequest, U256}, println, Address, Response};
use alloy_sol_types::{sol, SolCall, SolValue};
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


fn handle_message(_our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;

    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let body: serde_json::Value = serde_json::from_slice(message.body())?;
    println!("got {body:?}");
    Response::new()
        .body(serde_json::to_vec(&serde_json::json!("Ack")).unwrap())
        .send()
        .unwrap();
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("begin");

    let counter_address = EthAddress::from_str("0x1234567890123456789012345678901234567890").unwrap();

    let provider = Provider::new(31337, 5);
    let count = Counter::numberCall {}.abi_encode();

    let tx = TransactionRequest::default()
        .to(Some(counter_address))
        .input(TransactionInput::new(count.into()));
    let x = provider.call(tx, None);

    match x {
        Ok(b) => {
            let number = U256::abi_decode(&b, false).unwrap();
            println!("initial count: {:?}", number);
        }
        Err(e) => {
            println!("error getting initial count: {:?}", e);
        }
    }

    loop {
        match handle_message(&our) {
            Ok(()) => {}
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}
