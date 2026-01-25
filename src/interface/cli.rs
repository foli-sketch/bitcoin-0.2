use std::sync::{Arc, Mutex};

use crate::core::chain::Blockchain;
use crate::node::mempool::Mempool;
use crate::wallet::Wallet;
use crate::core::validation::validate_transaction;

/// CLI wallet & transaction commands
pub fn handle_command(
    args: Vec<String>,
    wallet: &mut Wallet,
    chain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
) {
    if args.len() < 3 {
        println!("Usage:");
        println!("  wallet balance");
        println!("  wallet send <to_pubkey_hash_hex> <amount>");
        return;
    }

    match args[2].as_str() {
        "balance" => {
            let chain_guard = chain.lock().unwrap();
            let my_hash = wallet.address().expect("wallet locked");

            let balance: u64 = chain_guard.utxos.values()
                .filter(|u| u.pubkey_hash == my_hash)
                .map(|u| u.value)
                .sum();

            println!("💰 Balance: {}", balance);
        }

        "send" => {
            if args.len() != 5 {
                println!("Usage: wallet send <to_pubkey_hash_hex> <amount>");
                return;
            }

            let to = match hex::decode(&args[3]) {
                Ok(v) => v,
                Err(_) => {
                    println!("Invalid pubkey hash");
                    return;
                }
            };

            let amount: u64 = match args[4].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("Invalid amount");
                    return;
                }
            };

            let chain_guard = chain.lock().unwrap();
            let current_height = chain_guard.height();

            let tx = match wallet.create_transaction(
                &chain_guard.utxos,
                to,
                amount,
            ) {
                Ok(t) => t,
                Err(e) => {
                    println!("❌ Wallet error: {}", e);
                    return;
                }
            };

            if !validate_transaction(&tx, &chain_guard.utxos, current_height) {
                println!("❌ Transaction failed consensus validation");
                return;
            }

            drop(chain_guard);

            let mut mempool_guard = mempool.lock().unwrap();
            let chain_guard = chain.lock().unwrap();

            if mempool_guard.add_transaction(tx, &chain_guard.utxos, current_height) {
                println!("✅ Transaction added to mempool");
            } else {
                println!("❌ Transaction rejected by mempool policy");
            }
        }

        _ => {
            println!("Unknown wallet command");
        }
    }
}
