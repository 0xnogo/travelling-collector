use ethers::prelude::*;

pub async fn get_block() {
    let ws_provider =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await
            .unwrap();

    let mut block_stream = ws_provider.subscribe_blocks().await.unwrap();

    println!("---------- MONITORING NEW BLOCKS ----------");
    while let Some(block) = block_stream.next().await {
        if let Some(hash_block) = block.hash {
            println!("{:?}", hash_block);
        }
    }
}

#[tokio::main]
async fn main() {
    get_block().await;
}
