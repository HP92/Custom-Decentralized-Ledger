use crate::models::Core;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[arg(short, long, value_name = "ADDRESS")]
    node: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    GenerateConfig {
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,
    },
}

pub async fn run_cli(core: Arc<Core>) -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "balance" => {
                log::info!("Current balance: {} satoshis", core.get_balance());
            }
            "send" => {
                if parts.len() != 3 {
                    log::warn!("Usage: send <recipient> <amount>");
                    continue;
                }
                let recipient = parts[1];
                let amount: u64 = parts[2].parse()?;
                let loaded_contact = core
                    .config()
                    .contacts()
                    .iter()
                    .find(|r| r.name() == recipient)
                    .ok_or_else(|| anyhow::anyhow!("Recipient not found"))?
                    .load()?;
                let recipient_key = loaded_contact.key();
                if let Err(e) = core.fetch_utxos().await {
                    log::error!("failed to fetch utxos: {e}");
                };
                let transaction = core.create_transaction(recipient_key, amount).await?;
                core.tx_sender().send(transaction).await?;
                log::info!("Transaction sent successfully");
                core.fetch_utxos().await?;
            }
            "exit" => break,
            _ => log::warn!("Unknown command"),
        }
    }
    Ok(())
}
