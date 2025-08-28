sudo apt install -y libpq-dev openssl libssl-dev pkg-config ca-certificates

curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh

cargo install cargo-watch
