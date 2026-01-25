use std::sync::{Arc, Mutex};

use crate::core::block::Block;
use crate::core::transaction::Transaction;
use crate::core::chain::Blockchain;
use crate::node::p2p::P2PNetwork as PrivateP2PNetwork;

pub struct P2PNetwork {
    inner: PrivateP2PNetwork,
}

impl P2PNetwork {
    pub fn new(chain: Arc<Mutex<Blockchain>>) -> Self {
        let inner = PrivateP2PNetwork::new(chain);
        Self { inner }
    }

    pub fn request_sync(&self) {
        self.inner.request_sync();
    }

    pub fn broadcast_block(&self, block: &Block) {
        self.inner.broadcast_block(block);
    }

    pub fn broadcast_transaction(&self, _tx: &Transaction) {
        // v0.3: transaction gossip not enabled yet
    }

    pub fn peer_count(&self) -> usize {
        self.inner.peer_count()
    }

    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.inner.local_addr()
    }
}
