use btclib::crypto::PrivateKey;
use btclib::utils::Saveable;
use clap::{Arg, Command};

pub fn main() {
    env_logger::init();

    let matches = Command::new("Key Generator")
        .version("1.0")
        .author("Charalampos Polychronakis <polychronakis.h@gmail.com>")
        .about("Generates a new public/private key pair and saves them to files")
        .arg(
            Arg::new("name")
                .help("Base name for the key files (e.g., 'mykey' creates 'mykey.pub.pem' and 'mykey.priv.cbor')")
                .required(true)
                .index(1),
        )
        .get_matches();

    let name = matches.get_one::<String>("name").unwrap();
    let private_key = PrivateKey::default();
    let public_key = private_key.public_key();
    let public_key_file = format!("{}.pub.pem", name);
    let private_key_file = format!("{}.priv.cbor", name);
    private_key.save_to_file(&private_key_file).unwrap();
    public_key.save_to_file(&public_key_file).unwrap();
}
