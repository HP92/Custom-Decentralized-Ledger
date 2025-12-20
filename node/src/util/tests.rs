#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_cli_default_port() {
        use clap::Parser;
        let cli = Cli::parse_from(&["node", "--blockchain-file", "test.cbor"]);
        assert_eq!(cli.port(), 9000);
    }

    #[test]
    fn test_cli_custom_port() {
        use clap::Parser;
        let cli = Cli::parse_from(&["node", "--blockchain-file", "test.cbor", "--port", "8080"]);
        assert_eq!(cli.port(), 8080);
    }

    #[test]
    fn test_cli_blockchain_file() {
        use clap::Parser;
        let cli = Cli::parse_from(&["node", "--blockchain-file", "my_blockchain.cbor"]);
        assert_eq!(cli.blockchain_file(), "my_blockchain.cbor");
    }

    #[test]
    fn test_cli_nodes_empty() {
        use clap::Parser;
        let cli = Cli::parse_from(&["node", "--blockchain-file", "test.cbor"]);
        assert!(cli.nodes().is_empty());
    }

    #[test]
    fn test_cli_nodes_single() {
        use clap::Parser;
        let cli = Cli::parse_from(&[
            "node",
            "--blockchain-file",
            "test.cbor",
            "--nodes",
            "localhost:9001",
        ]);
        assert_eq!(cli.nodes().len(), 1);
        assert_eq!(cli.nodes()[0], "localhost:9001");
    }

    #[test]
    fn test_cli_nodes_multiple() {
        use clap::Parser;
        let cli = Cli::parse_from(&[
            "node",
            "--blockchain-file",
            "test.cbor",
            "--nodes",
            "localhost:9001,localhost:9002,localhost:9003",
        ]);
        assert_eq!(cli.nodes().len(), 3);
        assert_eq!(cli.nodes()[0], "localhost:9001");
        assert_eq!(cli.nodes()[1], "localhost:9002");
        assert_eq!(cli.nodes()[2], "localhost:9003");
    }
}
