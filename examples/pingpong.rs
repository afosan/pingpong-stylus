use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use eyre::eyre;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;

/// Your private key file path.
const ENV_PRIV_KEY_PATH: &str = "PRIV_KEY_PATH";

/// Stylus RPC endpoint url.
const ENV_RPC_URL: &str = "RPC_URL";

/// Deployed pragram address.
const ENV_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();

    let priv_key_path = std::env::var(ENV_PRIV_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", ENV_PRIV_KEY_PATH))?;
    let rpc_url =
        std::env::var(ENV_RPC_URL).map_err(|_| eyre!("No {} env var set", ENV_RPC_URL))?;
    let program_address = std::env::var(ENV_PROGRAM_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", ENV_PROGRAM_ADDRESS))?;
    abigen!(
        PingPong,
        r#"[
            function init() external
            function pinger() external returns (address)
            function ping() external
            function pong(bytes32 tx_hash) external
        ]"#
    );

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let address: Address = program_address.parse()?;

    let privkey = read_secret_from_file(&priv_key_path)?;
    let wallet = LocalWallet::from_str(&privkey)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    let pingpong = PingPong::new(address, client);
    let pinger = pingpong.pinger().call().await?;
    println!("PingPong pinger = {:?}", pinger);

    if pinger == Address::zero() {
        let _ = pingpong.init().send().await?.log_msg("Send a tx to init contract").await?;
    }

    let pinger = pingpong.pinger().call().await?;
    println!("PingPong pinger = {:?}", pinger);

    if let Some(tx_receipt) = pingpong.ping().send().await?.log_msg("Send a tx to ping as pinger").await? {
        println!("Successfully emitted event {:?}", tx_receipt.logs[0]);
    } else {
        println!("Could not find tx receipt");
    }

    Ok(())
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
