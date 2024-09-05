use ckb_jsonrpc_types::{Script, TransactionView};
use ckb_types::H256;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::Server;
use jsonrpsee::types::ErrorObjectOwned;

use ssri_runner::types::{CellOutputWithData, Hex};
use ssri_runner::{SSRILevel, SSRIRunner};

#[rpc(server)]
pub trait Rpc {
    #[method(name = "run_script_level_code")]
    async fn run_script_level_code(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;

    #[method(name = "run_script_level_script")]
    async fn run_script_level_script(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        script: Script,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;

    #[method(name = "run_script_level_cell")]
    async fn run_script_level_cell(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        cell: CellOutputWithData,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;

    #[method(name = "run_script_level_tx")]
    async fn run_script_level_tx(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        tx: TransactionView,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;
}

pub struct RpcServerImpl {
    runner: SSRIRunner,
}

impl RpcServerImpl {
    pub fn new(rpc: &str) -> Self {
        Self {
            runner: SSRIRunner::new(rpc),
        }
    }
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    async fn run_script_level_code(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        println!("Received run_script_level_code request:");
        println!("  tx_hash: {:?}", tx_hash);
        println!("  index: {}", index);
        println!("  args: {:?}", args);

        self.runner
            .run_script(tx_hash, index, args, None, None, None)
            .await
    }

    async fn run_script_level_script(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        script: Script,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        self.runner
            .run_script_level_script(tx_hash, index, args, script)
            .await
    }

    async fn run_script_level_cell(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        cell: CellOutputWithData,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        self.runner
            .run_script_level_cell(tx_hash, index, args, cell)
            .await
    }

    async fn run_script_level_tx(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        tx: TransactionView,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        self.runner
            .run_script_level_tx(tx_hash, index, args, tx)
            .await
    }
}

pub async fn run_server(ckb_rpc: &str, server_addr: &str) -> anyhow::Result<()> {
    println!("Initializing server...");
    let server = Server::builder().build(server_addr).await?;
    println!("Server built successfully");

    let rpc_impl = RpcServerImpl::new(ckb_rpc);
    println!("RPC implementation created");

    let handle = server.start(rpc_impl.into_rpc());
    println!("Server started on {}", server_addr);

    println!("Waiting for Ctrl+C signal...");
    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl+C received, stopping server...");
    handle.stop().unwrap();
    println!("Server stopped");

    Ok(())
}
