use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use crate::core::chain::Blockchain;
use crate::node::p2p::P2PNetwork as InnerP2P;

/// Public-facing P2P wrapper (SEED SAFE)
pub struct P2PNetwork {
    inner: InnerP2P,
}

impl P2PNetwork {
    /// Fixed-port seed node
    pub fn bind(bind_addr: &str, chain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            inner: InnerP2P::bind(bind_addr, chain),
        }
    }

    /// Non-seed node (random port)
    pub fn new(chain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            inner: InnerP2P::new(chain),
        }
    }

    pub fn connect(&self, addr: SocketAddr) {
        self.inner.connect(addr);
    }

    pub fn peer_count(&self) -> usize {
        self.inner.peer_count()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.inner.local_addr()
    }
}
