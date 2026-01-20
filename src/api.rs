use axum::{
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use crate::chain::Blockchain;

#[derive(Serialize)]
struct StatusResponse {
    height: usize,
    difficulty: u32,
    utxos: usize,
}

#[derive(Serialize)]
struct BlockResponse {
    height: u64,
    hash: String,
    tx_count: usize,
    timestamp: i64,
    difficulty: u32,
}

pub async fn start_api(
    chain: Arc<Mutex<Blockchain>>,
    port: u16,
) {
    let app = Router::new()
        .route("/status", get({
            let chain = Arc::clone(&chain);
            move || async move {
                let chain = chain.lock().unwrap();
                Json(StatusResponse {
                    height: chain.blocks.len(),
                    difficulty: chain.difficulty,
                    utxos: chain.utxos.len(),
                })
            }
        }))
        .route("/blocks", get({
            let chain = Arc::clone(&chain);
            move || async move {
                let chain = chain.lock().unwrap();
                let blocks: Vec<BlockResponse> = chain.blocks.iter().map(|b| {
                    BlockResponse {
                        height: b.header.height,
                        hash: hex::encode(&b.hash),
                        tx_count: b.transactions.len(),
                        timestamp: b.header.timestamp,
                        difficulty: b.header.difficulty,
                    }
                }).collect();
                Json(blocks)
            }
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🌐 API running at http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app,
    )
    .await
    .unwrap();
}
