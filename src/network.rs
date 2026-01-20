use std::net::SocketAddr;
use crate::block::Block;
use crate::transaction::Transaction;
use crate::p2p::{P2PNetwork as PrivateP2PNetwork, PeerStats};

pub struct P2PNetwork {
    inner: PrivateP2PNetwork,
}

impl P2PNetwork {
    pub fn new(listen_addr: SocketAddr) -> Self {
        let network = PrivateP2PNetwork::new(listen_addr);
        network.start_listening();
        Self { inner: network }
    }

    pub fn with_seeds(listen_addr: SocketAddr, seed_nodes: Vec<SocketAddr>) -> Self {
        let network = PrivateP2PNetwork::discover_nodes(listen_addr, seed_nodes);
        network.start_listening();
        Self { inner: network }
    }

    pub fn broadcast_block(&self, block: &Block) {
        self.inner.broadcast_block(block);
    }

    pub fn broadcast_transaction(&self, tx: &Transaction) {
        self.inner.broadcast_transaction(tx);
    }

    pub fn request_sync(&self, from_height: u64) {
        self.inner.request_sync(from_height);
    }

    pub fn peer_count(&self) -> usize {
        self.inner.get_peer_count()
    }

    pub fn get_peers(&self) -> Vec<SocketAddr> {
        self.inner.get_peers()
    }

    pub fn get_stats(&self) -> PeerStats {
        self.inner.get_stats()
    }

    pub fn cleanup_stale_peers(&self) {
        self.inner.cleanup_stale_peers();
    }
}
