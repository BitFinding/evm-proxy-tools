use std::str::FromStr;

use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::BlockId;
use alloy_primitives::Address;
use clap::Parser;
use evm_proxy_tools::ProxyDispatch;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(value_parser = Address::from_str)]
    address: Address,

    #[clap(long, short)]
    block: Option<u64>,

    #[clap(short = 'r', long = "rpc-url", env = "ETH_RPC_URL")]
    pub url: String,
}

#[tokio::main]
async fn main() {
    let filter = EnvFilter::from_default_env();

    FmtSubscriber::builder()
        .with_env_filter(filter)
        .init();

    let args = Args::parse();

    println!("{:?}", args);

    let provider = ProviderBuilder::new()
        .connect_http(args.url.parse().expect("failed to parse RPC URL"));

    let mut address = args.address;

    loop {
        println!("Analysing address {:?}", address);
        
        let code = if let Some(block) = args.block {
            provider.get_code_at(address).block_id(BlockId::number(block)).await
        } else {
            provider.get_code_at(address).await
        }.expect("failed to find address at block");

        if code.is_empty() {
            println!("Address doesn't have a contract");
            std::process::exit(1);
        }

        let proxy_type = evm_proxy_tools::get_proxy_type(&code);

        println!("proxy type: {:?}", proxy_type);
        if let Some((_proxy_type, proxy_dispatch)) = proxy_type {
            if let ProxyDispatch::External(ext_address, _call) = proxy_dispatch {
                println!("going into proxy child");
                address = ext_address;
                continue;
            } else {
                let proxy_impl = evm_proxy_tools::get_proxy_implementation(
                    provider.clone(),
                    &address,
                    &proxy_dispatch,
                    args.block
                ).await.expect("failed to get proxy implementation");
                println!("proxy impl: {:?}", proxy_impl);
            }
        } else {
            println!("Couldn't identify a proxy in that address");
        }
        break;
    }
}
