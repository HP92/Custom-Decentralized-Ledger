use dashmap::DashMap;
use static_init::dynamic;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use btclib::types::Blockchain;

pub mod handler;
pub mod util;

#[dynamic]
pub static BLOCKCHAIN: RwLock<Blockchain> = RwLock::new(Blockchain::default());

#[dynamic]
pub static NODES: DashMap<String, TcpStream> = DashMap::new();
