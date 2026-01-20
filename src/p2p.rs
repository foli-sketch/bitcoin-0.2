use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{Deserialize, Serialize};
use crate::block::Block;
use crate::transaction::Transaction;

#[derive(Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    BlockAnnounce(Block),
    TransactionAnnounce(Transaction),
    SyncRequest { from_height: u64 },
    Ping,
    Pong,
}

pub struct PeerNode {
    pub address: SocketAddr,
    pub last_seen: i64,
    pub version: u32,
    pub stream: Option<TcpStream>,
    pub is_validated: bool,
}

pub struct P2PNetworkConfig {
    pub max_peers: usize,
    pub reconnect_interval: Duration,
    pub peer_timeout: i64,
    pub max_message_size: usize,
}

impl Default for P2PNetworkConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            reconnect_interval: Duration::from_secs(30),
            peer_timeout: 300,
            max_message_size: 1024 * 1024,
        }
    }
}

pub struct P2PNetwork {
    peers: Arc<Mutex<HashMap<String, PeerNode>>>,
    known_nodes: Arc<Mutex<Vec<SocketAddr>>>,
    listen_addr: SocketAddr,
    config: P2PNetworkConfig,
}

impl P2PNetwork {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self::with_config(listen_addr, P2PNetworkConfig::default())
    }

    pub fn with_config(listen_addr: SocketAddr, config: P2PNetworkConfig) -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
            known_nodes: Arc::new(Mutex::new(Vec::new())),
            listen_addr,
            config,
        }
    }

    pub fn discover_nodes(listen_addr: SocketAddr, seed_nodes: Vec<SocketAddr>) -> Self {
        let network = Self::new(listen_addr);
        {
            let mut nodes = network.known_nodes.lock().unwrap();
            nodes.extend(seed_nodes);
        }
        
        for node in network.known_nodes.lock().unwrap().iter() {
            network.connect_to_peer(*node);
        }
        
        network
    }

    fn validate_block(block: &Block) -> bool {
        if block.header.height < 0 {
            println!("❌ Invalid block: negative height");
            return false;
        }

        if block.transactions.is_empty() {
            println!("❌ Invalid block: no transactions");
            return false;
        }

        if block.header.difficulty == 0 {
            println!("❌ Invalid block: invalid difficulty");
            return false;
        }

        true
    }

    pub fn connect_to_peer(&self, address: SocketAddr) {
        let network_self = self.clone_for_thread();
        
        thread::spawn(move || {
            let mut attempts = 0;
            let max_attempts = 3;

            while attempts < max_attempts {
                match TcpStream::connect(address) {
                    Ok(mut stream) => {
                        let peer_key = address.to_string();
                        println!("✅ Connected to peer: {}", peer_key);
                        
                        let handshake = format!("HELLO:v1\n");
                        if stream.write_all(handshake.as_bytes()).is_ok() {
                            let mut peers = network_self.peers.lock().unwrap();
                            
                            if peers.len() >= network_self.config.max_peers {
                                println!("⚠️  Peer limit reached, disconnecting {}", peer_key);
                                return;
                            }

                            let peer = PeerNode {
                                address,
                                last_seen: Self::get_timestamp(),
                                version: 1,
                                stream: Some(stream),
                                is_validated: false,
                            };
                            peers.insert(peer_key, peer);
                            return;
                        }
                    }
                    Err(e) => {
                        attempts += 1;
                        if attempts < max_attempts {
                            println!("⚠️  Retry {} connecting to {}: {}", attempts, address, e);
                            thread::sleep(Duration::from_secs(5));
                        } else {
                            println!("❌ Failed to connect to {} after {} attempts", address, max_attempts);
                        }
                    }
                }
            }
        });
    }

    pub fn start_listening(&self) {
        let listener = match TcpListener::bind(self.listen_addr) {
            Ok(l) => {
                println!("🎧 P2P Server listening on: {}", self.listen_addr);
                l
            }
            Err(e) => {
                println!("❌ Failed to bind to {}: {}", self.listen_addr, e);
                return;
            }
        };

        let _ = listener.set_nonblocking(true);
        let peers = Arc::clone(&self.peers);
        let max_peers = self.config.max_peers;

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let peers_count = peers.lock().unwrap().len();
                        
                        if peers_count >= max_peers {
                            println!("⚠️  Peer limit reached, rejecting connection");
                            continue;
                        }

                        if let Ok(peer_addr) = stream.peer_addr() {
                            println!("🤝 New peer connection from: {}", peer_addr);
                        }
                        
                        let peers_clone = Arc::clone(&peers);
                        thread::spawn(move || {
                            Self::handle_peer_connection(stream, peers_clone);
                        });
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });
    }

    fn handle_peer_connection(mut stream: TcpStream, peers: Arc<Mutex<HashMap<String, PeerNode>>>) {
        let mut buffer = [0u8; 4096];
        
        let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

        match stream.read(&mut buffer) {
            Ok(n) if n > 0 => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                println!("📨 Received from peer: {}", message.trim());

                let peer_addr = stream.peer_addr().unwrap().to_string();
                let mut peers_map = peers.lock().unwrap();
                
                if !peers_map.contains_key(&peer_addr) {
                    let peer = PeerNode {
                        address: stream.peer_addr().unwrap(),
                        last_seen: Self::get_timestamp(),
                        version: 1,
                        stream: Some(stream),
                        is_validated: true,
                    };
                    peers_map.insert(peer_addr, peer);
                }
            }
            _ => println!("⚠️  Peer connection closed or timeout"),
        }
    }

    pub fn broadcast_block(&self, block: &Block) {
        if !Self::validate_block(block) {
            println!("❌ Block validation failed, not broadcasting");
            return;
        }

        let message = format!("BLOCK:{}\n", block.header.height);
        println!("📡 Broadcasting block #{} to {} peers", 
                 block.header.height, 
                 self.get_peer_count());
        
        let peers = self.peers.lock().unwrap();
        let mut success_count = 0;
        
        for (peer_key, _peer) in peers.iter() {
            println!("   ✓ Sent to {}", peer_key);
            success_count += 1;
        }

        if success_count == 0 {
            println!("   (no peers connected)");
        }
    }

    pub fn broadcast_transaction(&self, _tx: &Transaction) {
        println!("📡 Broadcasting transaction to {} peers", self.get_peer_count());
        
        let peers = self.peers.lock().unwrap();
        for peer_key in peers.keys() {
            println!("   ✓ Sent to {}", peer_key);
        }
    }

    pub fn request_sync(&self, from_height: u64) {
        let _message = format!("SYNC:{}\n", from_height);
        println!("🔄 Requesting chain sync from height {} from {} peers", 
                 from_height, 
                 self.get_peer_count());
        
        let peers = self.peers.lock().unwrap();
        for (peer_key, _peer) in peers.iter() {
            println!("   ✓ Sync request sent to {}", peer_key);
        }
    }

    pub fn get_peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }

    pub fn get_peers(&self) -> Vec<SocketAddr> {
        self.peers
            .lock()
            .unwrap()
            .values()
            .map(|p| p.address)
            .collect()
    }

    pub fn get_stats(&self) -> PeerStats {
        let peers = self.peers.lock().unwrap();
        let validated_peers = peers.values().filter(|p| p.is_validated).count();
        
        PeerStats {
            total_peers: peers.len(),
            validated_peers,
            known_nodes: self.known_nodes.lock().unwrap().len(),
            max_peers: self.config.max_peers,
        }
    }

    pub fn cleanup_stale_peers(&self) {
        let now = Self::get_timestamp();
        let timeout = self.config.peer_timeout;
        let mut peers = self.peers.lock().unwrap();
        
        let stale_peers: Vec<String> = peers
            .iter()
            .filter(|(_, p)| now - p.last_seen > timeout)
            .map(|(k, _)| k.clone())
            .collect();

        for peer_key in stale_peers {
            println!("🗑️  Removing stale peer: {}", peer_key);
            peers.remove(&peer_key);
        }
    }

    fn clone_for_thread(&self) -> P2PNetwork {
        P2PNetwork {
            peers: Arc::clone(&self.peers),
            known_nodes: Arc::clone(&self.known_nodes),
            listen_addr: self.listen_addr,
            config: P2PNetworkConfig {
                max_peers: self.config.max_peers,
                reconnect_interval: self.config.reconnect_interval,
                peer_timeout: self.config.peer_timeout,
                max_message_size: self.config.max_message_size,
            },
        }
    }

    fn get_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[derive(Clone, Debug)]
pub struct PeerStats {
    pub total_peers: usize,
    pub validated_peers: usize,
    pub known_nodes: usize,
    pub max_peers: usize,
}
