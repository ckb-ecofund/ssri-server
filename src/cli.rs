// a CLI that can run SSRI scripts and also start a RPC server to run scripts remotely.

use ckb_types::H256;
use ssri_runner::{types::Hex, SSRIRunner};
use std::str::FromStr;

mod server;

use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("SSRI CLI")
        .version("1.0")
        .about("CLI for executing SSRI scripts")
        .subcommand(
            Command::new("run")
                .about("Run a script")
                .arg(
                    Arg::new("tx_hash")
                        .long("tx-hash")
                        .required(true)
                        .help("Transaction hash"),
                )
                .arg(
                    Arg::new("index")
                        .long("index")
                        .required(true)
                        .help("Cell index"),
                )
                .arg(
                    Arg::new("ckb_rpc")
                        .long("ckb-rpc")
                        .help("CKB RPC URL")
                        .default_value("https://testnet.ckbapp.dev/"),
                )
                .arg(
                    Arg::new("args")
                        .action(ArgAction::Append)
                        .help("Script arguments"),
                ),
        )
        .subcommand(
            Command::new("server")
                .about("Start the RPC server")
                .arg(
                    Arg::new("ckb_rpc")
                        .long("ckb-rpc")
                        .help("CKB RPC URL")
                        .default_value("https://testnet.ckbapp.dev/"),
                )
                .arg(
                    Arg::new("server_addr")
                        .long("server-addr")
                        .help("Server address to listen on")
                        .default_value("localhost:9090"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("run", matches)) => {
            let tx_hash = matches
                .get_one::<String>("tx_hash")
                .expect("Transaction hash is required");
            let tx_hash = if let Some(stripped) = tx_hash.strip_prefix("0x") {
                H256::from_str(stripped)
            } else {
                H256::from_str(tx_hash)
            }
            .expect("Invalid transaction hash");
            let index = matches
                .get_one::<String>("index")
                .unwrap()
                .parse::<u32>()
                .expect("Invalid index");
            let args: Vec<Hex> = matches
                .get_many::<String>("args")
                .map(|values| values.map(|v| Hex::from(v.as_str())).collect())
                .unwrap_or_default();
            let ckb_rpc = matches
                .get_one::<String>("ckb_rpc")
                .unwrap_or(&"https://testnet.ckbapp.dev/".to_string())
                .to_string();

            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let runner = SSRIRunner::new(&ckb_rpc);
                    match runner
                        .run_script(tx_hash, index, args, None, None, None)
                        .await
                    {
                        Ok(result) => match result {
                            Some(hex) => println!("Script execution result: {:?}", hex.clone()),
                            None => println!("Script execution completed without a return value"),
                        },
                        Err(e) => eprintln!("Error executing script: {}", e),
                    }
                });
        }

        Some(("server", matches)) => {
            let ckb_rpc = matches
                .get_one::<String>("ckb_rpc")
                .unwrap_or(&"https://testnet.ckbapp.dev/".to_string())
                .to_string();
            let server_addr = matches
                .get_one::<String>("server_addr")
                .unwrap_or(&"0.0.0.0:9090".to_string())
                .to_string();

            tracing_subscriber::FmtSubscriber::builder()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .try_init()
                .expect("setting default subscriber failed");

            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    println!(
                        "Starting server with CKB RPC: {}, Server address: {}",
                        ckb_rpc, server_addr
                    );
                    match server::run_server(&ckb_rpc, &server_addr).await {
                        Ok(_) => println!("Server execution completed"),
                        Err(e) => eprintln!("Server error: {}", e),
                    }
                });
        }

        _ => unreachable!(),
    }
}
