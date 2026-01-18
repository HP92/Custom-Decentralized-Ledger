use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value_t = 9000)]
    port: u16,

    /// Path to the blockchain file
    #[arg(short, long)]
    blockchain_file: String,

    /// List of peer nodes
    #[arg(short, long, value_delimiter = ',')]
    nodes: Vec<String>,
}

impl Cli {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn blockchain_file(&self) -> &str {
        &self.blockchain_file
    }

    pub fn nodes(&self) -> &Vec<String> {
        &self.nodes
    }
}
