use crate::models::{Config, FeeConfig, FeeType, Recipient};
use anyhow::Result;
use cursive::reexports::serde_json;
use std::path::PathBuf;

pub fn generate_dummy_config(path: &PathBuf) -> Result<()> {
    let alice = Recipient::new("Alice".to_string(), PathBuf::from("alice.pub.pem"));
    let bob = Recipient::new("Bob".to_string(), PathBuf::from("bob.pub.pem"));

    let dummy_fee_config = FeeConfig::new(FeeType::Percent, 0.1);

    let config = Config::new(
        vec![],
        vec![alice.clone(), bob.clone()],
        "127.0.0.1:9000".to_string(),
        dummy_fee_config,
    );

    let config_data = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, config_data)?;

    log::info!("Dummy config generated at {:?}", path);
    Ok(())
}
