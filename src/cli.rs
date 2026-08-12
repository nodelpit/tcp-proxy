use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser)]
pub struct Config {
    #[arg(long)]
    pub listener: SocketAddr,

    #[arg(long)]
    pub target: SocketAddr,
}

#[cfg(test)]
mod test {
    use super::Config;
    use clap::Parser;

    #[test]
    fn parses_valid_ipv4_addresses() {
        let args = [
            "tcp-proxy",
            "--listener",
            "127.0.0.1:8080",
            "--target",
            "127.0.0.1:9000",
        ];

        let config = Config::try_parse_from(args).unwrap();

        assert_eq!(config.listener, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(config.target, "127.0.0.1:9000".parse().unwrap());
    }

    #[test]
    fn parses_valid_ipv6_addresses() {
        let args = [
            "tcp-proxy",
            "--listener",
            "[::1]:8080",
            "--target",
            "[::1]:9000",
        ];

        let config = Config::try_parse_from(args).unwrap();

        assert_eq!(config.listener, "[::1]:8080".parse().unwrap());
        assert_eq!(config.target, "[::1]:9000".parse().unwrap());
    }

    #[test]
    fn rejects_malformed_address() {
        let args = [
            "tcp-proxy",
            "--listener",
            "127.0.0.1:17.77",
            "--target",
            "127.0.0.1:9000",
        ];

        let config = Config::try_parse_from(args);

        assert!(config.is_err())
    }

    #[test]
    fn rejects_missing_required_argument() {
        let args = ["tcp-proxy", "--listener", "127.0.0.1:8080"];

        let config = Config::try_parse_from(args);
        assert!(config.is_err())
    }
}
