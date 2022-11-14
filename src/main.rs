mod block_scanner;
mod contract;
mod helper;
mod report;
mod source_code;
mod vulnerability;

use ethers::providers::{Middleware, Provider, Ws};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mode: bool = match &args[1][..] {
        "backwards" => true,
        _ => false,
    }; // true
    let path_to_file = &args[2].to_owned();

    let ws_provider = Provider::<Ws>::connect(env::var("RPC_WS_ENDPOINT").unwrap())
        .await
        .unwrap();

    let latest_block_number = ws_provider.get_block_number().await.unwrap();

    if mode {
        let mut input_block: u64 = args[3].parse().unwrap(); // 15900000
        loop {
            let block = ws_provider.get_block(input_block).await.unwrap().unwrap();
            block_scanner::analyze_block(&ws_provider, block, latest_block_number, path_to_file)
                .await;
            if input_block == 0 {
                break;
            }
            input_block -= 1;
        }
    } else {
        block_scanner::monitoring_blocks(&ws_provider, latest_block_number, path_to_file).await;
    }
}
