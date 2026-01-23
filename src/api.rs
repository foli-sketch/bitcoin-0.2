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
    response::{Html, IntoResponse},
};

use crate::chain::Blockchain;
use crate::wallet::Wallet;

/* -------------------- APP STATE -------------------- */

#[derive(Clone)]
struct AppState {
    chain: Arc<Mutex<Blockchain>>,
}

/* -------------------- START API -------------------- */

/// HTTP API / Explorer
///
/// ⚠️ READ-ONLY except for `/send`
/// `/send` uses a DEV faucet wallet and must NOT be enabled in production
pub async fn start_api(chain: Arc<Mutex<Blockchain>>, port: u16) {
    let state = AppState { chain };

    let app = Router::new()
        // JSON
        .route("/blocks", get(blocks))
        .route("/block/height/:height", get(block_by_height))
        .route("/tx/:txid", get(tx_by_id))
        .route("/address/:hash", get(address_info))

        // DEV ONLY
        .route("/send", post(send_tx))

        // HTML
        .route("/", get(index_html))
        .route("/blocks.html", get(blocks_html))
        .route("/block/:height", get(block_html))
        .route("/tx.html/:txid", get(tx_html))
        .route("/address.html/:hash", get(address_html))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🌐 Explorer running at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/* -------------------- JSON -------------------- */

#[derive(Serialize)]
struct BlockResponse {
    height: u64,
    hash: String,
    txs: usize,
}

async fn blocks(
    State(state): State<AppState>,
) -> Json<Vec<BlockResponse>> {
    let c = state.chain.lock().unwrap();

    Json(
        c.blocks
            .iter()
            .map(|b| BlockResponse {
                height: b.header.height,
                hash: hex(&b.hash),
                txs: b.transactions.len(),
            })
            .collect(),
    )
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
        })
        .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

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
                })
                .into_response();
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

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

    Json(AddressResponse {
        balance,
        utxos: count,
    })
}

/* -------------------- DEV SEND (FAUCET) -------------------- */

#[derive(Deserialize)]
struct SendRequest {
    to: String,
    amount: u64,
    fee: u64,
}

#[derive(Serialize)]
struct SendResponse {
    status: String,
    txid: String,
}

/// ⚠️ DEV FAUCET ONLY
async fn send_tx(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> impl IntoResponse {
    let chain = state.chain.lock().unwrap();

    let wallet = Wallet::new_dev();

    let to = match hex::decode(req.to) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let tx = wallet.send(
        &chain.utxos,
        to,
        req.amount,
        req.fee,
    );

    Json(SendResponse {
        status: "ok".to_string(),
        txid: hex(&tx.txid()),
    })
    .into_response()
}

/* -------------------- HTML -------------------- */

async fn index_html() -> Html<String> {
    Html("<h1>Rust Bitcoin Explorer</h1>".to_string())
}

async fn blocks_html(
    State(state): State<AppState>,
) -> Html<String> {
    let c = state.chain.lock().unwrap();
    let mut rows = String::new();

    for b in &c.blocks {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            b.header.height,
            hex(&b.hash),
            b.transactions.len()
        ));
    }

    Html(format!(
        "<table border='1'>
            <tr><th>Height</th><th>Hash</th><th>Txs</th></tr>
            {}
        </table>",
        rows
    ))
}

async fn block_html(
    State(state): State<AppState>,
    Path(height): Path<u64>,
) -> impl IntoResponse {
    let c = state.chain.lock().unwrap();

    match c.blocks.iter().find(|b| b.header.height == height) {
        Some(b) => Html(format!(
            "<h2>Block #{}</h2><p>{}</p>",
            height,
            hex(&b.hash)
        ))
        .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn tx_html(
    State(state): State<AppState>,
    Path(txid): Path<String>,
) -> impl IntoResponse {
    let c = state.chain.lock().unwrap();

    for block in &c.blocks {
        for tx in &block.transactions {
            if hex(&tx.txid()) == txid {
                return Html(format!(
                    "<p>TxID: {}</p>",
                    txid
                ))
                .into_response();
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn address_html(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Html<String> {
    let c = state.chain.lock().unwrap();
    let mut balance = 0;

    for u in c.utxos.values() {
        if hex(&u.pubkey_hash) == hash {
            balance += u.value;
        }
    }

    Html(format!(
        "<p>{}</p><p>Balance: {}</p>",
        hash,
        balance
    ))
}

/* -------------------- UTIL -------------------- */

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
