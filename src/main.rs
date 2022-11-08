mod block_scanner;
mod contract;
mod helper;

use ethers::providers::{Middleware, Provider, Ws};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input_block: u64 = args[1].parse().unwrap(); // 15900000
    let backwards: bool = match &args[2][..] {
        "true" => true,
        _ => false,
    }; // true

    let ws_provider = Provider::<Ws>::connect(env::var("RPC_WS_ENDPOINT").unwrap())
        .await
        .unwrap();

    let latest_block_number = ws_provider.get_block_number().await.unwrap();

    if backwards {
        loop {
            let block = ws_provider.get_block(input_block).await.unwrap().unwrap();
            println!("Perform logic on {} block", input_block);
            block_scanner::analyze_block(&ws_provider, block, latest_block_number).await;
            if input_block == 0 {
                break;
            }
            input_block -= 1;
        }
    } else {
        block_scanner::monitoring_blocks(&ws_provider, latest_block_number).await;
    }
}
