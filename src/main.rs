use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use tokio::runtime::Runtime;
use rpassword::read_password;

// Updated imports to match your new package name
use bitcoin_0_2::core::chain::Blockchain;
use bitcoin_0_2::node::p2p::P2PNetwork;
use bitcoin_0_2::node::transport::tcp::TcpTransport;
use bitcoin_0_2::node::mempool::Mempool;
use bitcoin_0_2::wallet::Wallet;
use bitcoin_0_2::wallet_store::load_wallet_store;
use bitcoin_0_2::config::load_miner_config;
use bitcoin_0_2::node::miner;

fn prompt_secret(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    read_password().unwrap()
}

fn main() {
    println!("Bitcoin Node v0.3.3 Starting...");

    let _wallet_store = load_wallet_store();
    let _miner_config = load_miner_config();

    let password = prompt_secret("Enter wallet password: ");
    let mut wallet = Wallet::load_or_create(&password);
    let miner_pubkey_hash = wallet.address().expect("wallet locked");

    let mut local_chain = Blockchain::new();
    local_chain.initialize();

    let chain = Arc::new(Mutex::new(local_chain));
    let mempool = Arc::new(Mutex::new(Mempool::new()));

    let transport = Arc::new(TcpTransport::new());
    let p2p = Arc::new(P2PNetwork::new(Arc::clone(&transport), Arc::clone(&chain)));

    thread::spawn(move || {
        println!("P2P Network initialized.");
    });

    loop {
        let txs = mempool.lock().unwrap().sorted_for_mining();
        let candidate_block = {
            let c = chain.lock().unwrap();
            let prev = c.blocks.last().unwrap();
            miner::mine_block(prev, &c.utxos, txs, miner_pubkey_hash.clone(), &c.blocks)
        };

        let accepted = {
            let mut c = chain.lock().unwrap();
            c.validate_and_add_block(candidate_block.clone())
        };

        if accepted {
            p2p.broadcast(&candidate_block);
            mempool.lock().unwrap().remove_confirmed(&candidate_block.transactions);
            println!("New block accepted!");
        }
        thread::sleep(Duration::from_millis(1000));
    }
}
