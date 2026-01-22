use crate::chain::Blockchain;

fn display_chain(blockchain: &Blockchain) {
    println!("\n📊 Blockchain Status:");
    println!("Height: {}", blockchain.blocks.len());
    println!("Difficulty: {}", blockchain.difficulty);
    println!("UTXO Set Size: {}", blockchain.utxos.len());
}

fn display_block_info(blockchain: &Blockchain) {
    if let Some(latest) = blockchain.blocks.last() {
        println!("\n🔗 Latest Block:");
        println!("Height: {}", latest.header.height);
        println!("Transactions: {}", latest.transactions.len());
        println!("Timestamp: {}", latest.header.timestamp);
    }
}

pub fn display_full_chain(blockchain: &Blockchain) {
    display_chain(blockchain);
    display_block_info(blockchain);
}
