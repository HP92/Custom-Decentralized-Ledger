use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use wallet::{
    models::{Commands, Core},
    util::{
        Cli, generate_dummy_config, handle_transactions, run_cli, update_utxos,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::GenerateConfig { output }) => {
            return generate_dummy_config(output);
        }
        None => {}
    }
    let config_path = cli
        .config
        .unwrap_or_else(|| PathBuf::from("wallet_config.toml"));
    let mut core = Core::load(config_path.clone())?;
    if let Some(node) = cli.node {
        core.config.default_node = node;
    }
    let (tx_sender, tx_receiver) = kanal::bounded(10);
    core.tx_sender = tx_sender.clone_async();
    let core = Arc::new(core);
    tokio::spawn(update_utxos(core.clone()));
    tokio::spawn(handle_transactions(tx_receiver.clone_async(), core.clone()));
    run_cli(core).await?;
    Ok(())
}
