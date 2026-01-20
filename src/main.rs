use bitcoin_v0_2_revelation::chain::Blockchain;
use bitcoin_v0_2_revelation::network::P2PNetwork;
use std::net::SocketAddr;

fn print_chain(chain: &Blockchain) {
    println!("\n🔗 Full Blockchain:");
    println!("═══════════════════════════════════════");
    for block in &chain.blocks {
        println!("Block #{}", block.header.height);
        println!("  Hash: {}", hex::encode(&block.hash));
        println!("  Transactions: {}", block.transactions.len());
        println!("  Timestamp: {}", block.header.timestamp);
        println!("  Difficulty: {}", block.header.difficulty);
        println!("─────────────────────────────────────");
    }
}

fn main() {
    println!("⛓ Bitcoin v0.2 — Revelation Edition");

    let mut chain = Blockchain::new();
    chain.initialize();

    let miner_key = "REVELATION_MINER_001";

    // P2P Configuration - add seed node addresses here to connect to peers
    let listen_addr = "127.0.0.1:8333".parse::<SocketAddr>().unwrap();
    let seed_nodes: Vec<SocketAddr> = vec![
        // Example seed nodes - modify these to connect to actual peers
        // "192.168.1.100:8333".parse::<SocketAddr>().unwrap(),
        // "192.168.1.101:8333".parse::<SocketAddr>().unwrap(),
        // "10.0.0.50:8333".parse::<SocketAddr>().unwrap(),
        // For local testing, uncomment and use different ports:
        // "127.0.0.1:8334".parse::<SocketAddr>().unwrap(),
        // "127.0.0.1:8335".parse::<SocketAddr>().unwrap(),
    ];

    // Initialize P2P network
    let p2p = P2PNetwork::with_seeds(listen_addr, seed_nodes);
    println!("🌐 P2P Network initialized (listen on {})", listen_addr);
    println!("📡 Connected to {} peer(s)", p2p.peer_count());

    let mut block_count = 0;
    loop {
        chain.mine_block(miner_key);
        block_count += 1;

        // Broadcast mined block to peers every block
        if let Some(latest) = chain.blocks.last() {
            p2p.broadcast_block(latest);
        }

        // Display chain status every 5 blocks
        if block_count % 5 == 0 {
            println!("\n📊 Blockchain Status:");
            println!("Height: {}", chain.blocks.len());
            println!("Difficulty: {}", chain.difficulty);
            println!("UTXO Set Size: {}", chain.utxos.len());
            println!("Connected Peers: {}", p2p.peer_count());
            
            if let Some(latest) = chain.blocks.last() {
                println!("Latest Block Height: {}", latest.header.height);
                println!("Latest Block Transactions: {}", latest.transactions.len());
            }

            // Show network stats every 10 blocks
            if block_count % 10 == 0 {
                let stats = p2p.get_stats();
                println!("\n🌐 Network Stats:");
                println!("  Total Peers: {}", stats.total_peers);
                println!("  Validated Peers: {}", stats.validated_peers);
                println!("  Known Nodes: {}", stats.known_nodes);
                println!("  Max Peers: {}", stats.max_peers);
            }

            // Cleanup stale peers every 20 blocks
            if block_count % 20 == 0 {
                p2p.cleanup_stale_peers();
                print_chain(&chain);
            }
        }
    }
}
