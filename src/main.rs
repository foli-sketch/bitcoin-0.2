use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use bitcoin_v0_2_revelation::core::chain::Blockchain;
use bitcoin_v0_2_revelation::node::network::P2PNetwork;

fn main() {
    println!("🌱 Bitcoin v0.3.2 — Revelation Edition");
    println!("🌍 Seed Node (Consensus v3)");

    let mut chain = Blockchain::new();
    chain.initialize();

    let chain = Arc::new(Mutex::new(chain));

    // 🔥 FIXED PORT SEED
    let p2p = P2PNetwork::bind("0.0.0.0:8333", Arc::clone(&chain));

    println!("🔗 Seed listening on {}", p2p.local_addr());

    loop {
        println!("🌐 peers={}", p2p.peer_count());
        thread::sleep(Duration::from_secs(30));
    }
}
