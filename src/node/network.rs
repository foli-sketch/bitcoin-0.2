use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use crate::core::block::Block;
use crate::core::transaction::Transaction;
use crate::core::chain::Blockchain;

// Internal P2P implementation
use crate::node::p2p::P2PNetwork as InnerP2P;

/// Public-facing P2P wrapper
/// This layer exposes ONLY safe, non-consensus APIs
pub struct P2PNetwork {
    inner: InnerP2P,
}

impl P2PNetwork {
    /// Random-port node (non-seed)
    pub fn new(chain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            inner: InnerP2P::new(chain),
        }
    }

    /// Fixed-port node (SEED)
    pub fn bind(bind_addr: &str, chain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            inner: InnerP2P::bind(bind_addr, chain),
        }
    }

    pub fn connect(&self, addr: SocketAddr) {
        self.inner.connect(addr);
    }

    pub fn broadcast_block(&self, block: &Block) {
        self.inner.broadcast_block(block);
    }

    pub fn broadcast_transaction(&self, tx: &Transaction) {
        self.inner.broadcast_transaction(tx);
    }

    pub fn peer_count(&self) -> usize {
        self.inner.peer_count()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.inner.local_addr()
    }
}
