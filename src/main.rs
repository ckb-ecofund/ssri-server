use ckb_types::H256;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::Server;
use jsonrpsee::tracing;
use jsonrpsee::types::ErrorObjectOwned;

mod error;
mod rpc_client;
mod ssri_vm;
mod types;

use error::Error;

use rpc_client::RpcClient;
use types::Hex;

use ckb_jsonrpc_types::OutPoint;

use ssri_vm::execute_riscv_binary;

#[rpc(server)]
pub trait Rpc {
    #[method(name = "run_script")]
    async fn run_script(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;
}

pub struct RpcServerImpl {
    rpc: RpcClient,
}

impl RpcServerImpl {
    pub fn new() -> Self {
        Self {
            rpc: RpcClient::new("https://testnet.ckbapp.dev/"),
        }
    }
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    async fn run_script(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        let cell = self
            .rpc
            .get_live_cell(
                &OutPoint {
                    tx_hash: tx_hash.0.into(),
                    index: index.into(),
                },
                true,
            )
            .await?;

        tracing::info!("Running script on {tx_hash}:{index} with args {args:?}");

        let data = cell
            .cell
            .ok_or(Error::InvalidRequest("Cell not found"))?
            .data
            .ok_or(Error::InvalidRequest("Cell doesn't have data"))?
            .content
            .into_bytes();

        Ok(
            execute_riscv_binary(data, args.into_iter().map(|v| v.hex.into()).collect())?
                .map(|v| v.into()),
        )
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    run_server().await?;
    Ok(())
}

async fn run_server() -> anyhow::Result<()> {
    let server = Server::builder().build("127.0.0.1:8090").await?;

    let handle = server.start(RpcServerImpl::new().into_rpc());

    tokio::signal::ctrl_c().await.unwrap();
    handle.stop().unwrap();

    Ok(())
}
