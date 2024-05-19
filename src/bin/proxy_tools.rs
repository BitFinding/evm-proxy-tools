use std::{str::FromStr, sync::Arc};

use clap::Parser;
use ethers_core::{types::{NameOrAddress, BlockId}, macros::ethers_providers_crate};
use ethers_providers::{JsonRpcClient, Http, Middleware, Provider};
use evm_proxy_tools::{ProxyType, ProxyDispatch};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use evm_proxy_tools::utils::EARGlue;


/// A `clap` `value_parser` that removes a `0x` prefix if it exists
pub fn strip_0x_prefix(s: &str) -> Result<String, &'static str> {
    Ok(s.strip_prefix("0x").unwrap_or(s).to_string())
}

/// CLI arguments for `proxy-tools`.
#[command(author, version, about, long_about = None)]
// #[command(
//     help_template = "{author-with-newline} {about-section}Version: {version} \n {usage-heading} {usage} \n {all-args} {tab}"
// )]
#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// The contract address.
    #[clap(value_parser = NameOrAddress::from_str)]
    address: NameOrAddress,

    /// The block height to query at.
    ///
    /// Can also be the tags earliest, finalized, safe, latest, or pending.
    #[clap(long, short)]
    block: Option<BlockId>,

    /// The RPC endpoint.
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

    // let url = Url::from(args.url).unwrap();
    let rpc = Arc::new(Provider::<Http>::try_from(&args.url).expect("failed to create rpc connection with url"));
    // let code = rpc.get_code(args.address, args.block).await;

    let mut address = args.address.clone();

    loop {
	println!("Analysing address {:?}", address.as_address().unwrap());

	let rpc = rpc.clone();
	let code = rpc.get_code(address.clone(), args.block).await.expect("failed to find address at block");
	// println!("code: {:?}", code);

	if code.is_empty() {
	    println!("Address doesn't have a contract");
	    std::process::exit(1);
	}

	let proxy_type = evm_proxy_tools::get_proxy_type(&code);

	println!("proxy type: {:?}", proxy_type);
	if let Some((proxy_type, proxy_dispatch)) = proxy_type {
	    if let ProxyDispatch::External(ext_address, call) = proxy_dispatch {
		println!("going into proxy child");
		address = ext_address.convert();
		continue;
	    } else {
		let raddress = evm_proxy_tools::utils::h160_to_b160(&address.as_address().unwrap());
		let proxy_impl = evm_proxy_tools::get_proxy_implementation(rpc, &raddress, &proxy_dispatch).await.expect("somehow failed to");
		println!("proxy impl: {:?}", proxy_impl);
	    }
	} else {
	    println!("Couldn't identify a proxy in that address");
	}
	break;
    }
}
