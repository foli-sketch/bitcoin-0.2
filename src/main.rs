use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::env;

use tokio::runtime::Runtime;

// Import from the LIB crate (not crate::)
use bitcoin_v0_2_revelation::core::chain::Blockchain;
use bitcoin_v0_2_revelation::node::network::P2PNetwork;
use bitcoin_v0_2_revelation::interface::{api::start_api, cli};
use bitcoin_v0_2_revelation::node::mempool::Mempool;
use bitcoin_v0_2_revelation::wallet::Wallet;
use bitcoin_v0_2_revelation::wallet_store::load_wallet_store;
use bitcoin_v0_2_revelation::config::load_miner_config;
use bitcoin_v0_2_revelation::node::miner;

enum NodeMode {
    Syncing,
    Normal,
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn main() {
    println!("⛓ Bitcoin v0.3.0 — Revelation Edition (Consensus v3)");

    let wallet_store = load_wallet_store();
    let miner_config = load_miner_config();

    if wallet_store.get_path(&miner_config.coinbase_wallet).is_none() {
        panic!("Configured wallet '{}' not found", miner_config.coinbase_wallet);
    }

    let _passphrase = prompt("🔐 Enter wallet passphrase: ");
    let password   = prompt("🔑 Enter wallet password: ");

    let mut wallet = Wallet::load_or_create(&password);
    let miner_pubkey_hash = wallet.address().expect("wallet locked");

    println!(
        "👛 Miner pubkey hash ({}): {}",
        miner_config.coinbase_wallet,
        hex::encode(&miner_pubkey_hash)
    );

    // ───────── Blockchain ─────────
    let mut local_chain = Blockchain::new();
    local_chain.initialize();

    let chain = Arc::new(Mutex::new(local_chain));
    let mempool = Arc::new(Mutex::new(Mempool::new()));

    // ───────── CLI MODE (early exit) ─────────
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "wallet" {
        cli::handle_command(
            args,
            &mut wallet,
            Arc::clone(&chain),
            Arc::clone(&mempool),
        );
        return;
    }

    // ───────── API Server ─────────
    let api_chain = Arc::clone(&chain);
    thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(start_api(api_chain, 8080));
    });

    println!("🌐 Explorer running at http://127.0.0.1:8080");

    // ───────── P2P ─────────
    let p2p = P2PNetwork::new(Arc::clone(&chain));
    println!("🌐 P2P listening on {}", p2p.local_addr());

    let mut mode = NodeMode::Syncing;
    let mut last_height = chain.lock().unwrap().height();
    let mut last_change = Instant::now();
    let mut last_balance: u64 = 0;

    println!("🔄 Requesting sync from peers");
    p2p.request_sync();

    loop {
        match mode {
            NodeMode::Syncing => {
                let height = chain.lock().unwrap().height();

                if height != last_height {
                    last_height = height;
                    last_change = Instant::now();
                }

                if last_change.elapsed() > Duration::from_secs(3) && height > 0 {
                    println!("✅ Sync complete at height {}", height);
                    mode = NodeMode::Normal;
                }

                sleep(Duration::from_millis(300));
            }

            NodeMode::Normal => {
                let mempool_txs = mempool.lock().unwrap().sorted_for_mining();

                let candidate_block = {
                    let chain_guard = chain.lock().unwrap();
                    let prev_block = chain_guard.blocks.last().unwrap();

                    miner::mine_block(
                        prev_block,
                        &chain_guard.utxos,
                        mempool_txs,
                        miner_pubkey_hash.clone(),
                        &chain_guard.blocks,
                    )
                };

                let accepted = {
                    let mut chain_guard = chain.lock().unwrap();
                    chain_guard.validate_and_add_block(candidate_block.clone())
                };

                if accepted {
                    p2p.broadcast_block(&candidate_block);

                    mempool.lock().unwrap()
                        .remove_confirmed(&candidate_block.transactions);

                    let chain_guard = chain.lock().unwrap();
                    let balance = chain_guard.utxos.values()
                        .filter(|u| u.pubkey_hash == miner_pubkey_hash)
                        .map(|u| u.value)
                        .sum::<u64>();

                    let height = chain_guard.height();

                    if balance != last_balance {
                        println!("💰 Wallet balance: {} (height {})", balance, height);
                        last_balance = balance;
                    }
                }

                sleep(Duration::from_millis(100));
            }
        }
    }
}
