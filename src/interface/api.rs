use tokio::net::TcpListener;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use axum::{
    Router,
    Json,
    routing::{get, post},
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::chain::Blockchain;

#[derive(Clone)]
struct AppState {
    chain: Arc<Mutex<Blockchain>>,
}

pub async fn start_api(chain: Arc<Mutex<Blockchain>>, port: u16) {
    let state = AppState { chain };

    let app = Router::new()
        .route("/status", get(status))
        .route("/blocks", get(blocks))
        .route("/block/height/:height", get(block_by_height))
        .route("/tx/:txid", get(tx_by_id))
        .route("/address/:hash", get(address_info))
        .route("/transactions/new", post(new_transaction)) // 🔥 NEW
        .with_state(state);

    // 🔥 IMPORTANT: allow connections from your phone / LAN
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

//
// ─── STATUS ─────────────────────────────────────────
//

#[derive(Serialize)]
struct StatusResponse {
    height: u64,
    blocks: usize,
    utxos: usize,
    mempool: usize,
}

async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let c = state.chain.lock().unwrap();
    Json(StatusResponse {
        height: c.height(),
        blocks: c.blocks.len(),
        utxos: c.utxos.len(),
        mempool: c.mempool.len(),
    })
}

//
// ─── BLOCKS ─────────────────────────────────────────
//

#[derive(Serialize)]
struct BlockResponse {
    height: u64,
    hash: String,
    txs: usize,
}

async fn blocks(State(state): State<AppState>) -> Json<Vec<BlockResponse>> {
    let c = state.chain.lock().unwrap();
    Json(c.blocks.iter().map(|b| BlockResponse {
        height: b.header.height,
        hash: hex(&b.hash),
        txs: b.transactions.len(),
    }).collect())
}

async fn block_by_height(
    State(state): State<AppState>,
    Path(height): Path<u64>,
) -> impl IntoResponse {
    let c = state.chain.lock().unwrap();
    match c.blocks.iter().find(|b| b.header.height == height) {
        Some(b) => Json(BlockResponse {
            height,
            hash: hex(&b.hash),
            txs: b.transactions.len(),
        }).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

//
// ─── TRANSACTIONS ───────────────────────────────────
//

#[derive(Serialize)]
struct TxResponse {
    txid: String,
    inputs: usize,
    outputs: usize,
}

async fn tx_by_id(
    State(state): State<AppState>,
    Path(txid): Path<String>,
) -> impl IntoResponse {
    let c = state.chain.lock().unwrap();
    for block in &c.blocks {
        for tx in &block.transactions {
            if hex(&tx.txid()) == txid {
                return Json(TxResponse {
                    txid,
                    inputs: tx.inputs.len(),
                    outputs: tx.outputs.len(),
                }).into_response();
            }
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

//
// ─── NEW TRANSACTION (MEMPOOL) ─────────────────────
//

#[derive(Deserialize)]
struct NewTxRequest {
    from: String, // hex pubkey_hash
    to: String,   // hex pubkey_hash
    amount: u64,
}

async fn new_transaction(
    State(state): State<AppState>,
    Json(req): Json<NewTxRequest>,
) -> impl IntoResponse {
    let mut chain = state.chain.lock().unwrap();

    let from = match hex::decode(&req.from) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid sender").into_response(),
    };

    let to = match hex::decode(&req.to) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid receiver").into_response(),
    };

    match chain.create_transaction(from, to, req.amount) {
        Ok(tx) => {
            let txid = hex(&tx.txid());
            chain.mempool.push(tx);
            (
                StatusCode::OK,
                format!("Transaction added to mempool: {}", txid),
            ).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Transaction failed: {}", e),
        ).into_response(),
    }
}

//
// ─── ADDRESS INFO ───────────────────────────────────
//

#[derive(Serialize)]
struct AddressResponse {
    balance: u64,
    utxos: usize,
}

async fn address_info(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Json<AddressResponse> {
    let c = state.chain.lock().unwrap();
    let mut balance = 0;
    let mut count = 0;

    for u in c.utxos.values() {
        if hex(&u.pubkey_hash) == hash {
            balance += u.value;
            count += 1;
        }
    }

    Json(AddressResponse { balance, utxos: count })
}

//
// ─── HELPER ─────────────────────────────────────────
//

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
