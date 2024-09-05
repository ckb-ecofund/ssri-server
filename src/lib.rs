use ckb_jsonrpc_types::{OutPoint, Script, TransactionView};
use ckb_types::H256;
use jsonrpsee::core::async_trait;
use jsonrpsee::tracing;
use jsonrpsee::types::ErrorObjectOwned;

pub mod error;
pub mod rpc_client;
pub mod ssri_vm;
pub mod types;

use error::Error;
use rpc_client::RpcClient;
use ssri_vm::execute_riscv_binary;
use types::{CellOutputWithData, Hex};

#[async_trait]
pub trait SSRILevel {
    async fn run_script_level_code(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;

    async fn run_script_level_script(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        script: Script,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;

    async fn run_script_level_cell(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        cell: CellOutputWithData,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;

    async fn run_script_level_tx(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        tx: TransactionView,
    ) -> Result<Option<Hex>, ErrorObjectOwned>;
}

pub struct SSRIRunner {
    rpc: RpcClient,
}

impl SSRIRunner {
    pub fn new(rpc: &str) -> Self {
        Self {
            rpc: RpcClient::new(rpc),
        }
    }

    pub async fn run_script(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        script: Option<Script>,
        cell: Option<CellOutputWithData>,
        tx: Option<TransactionView>,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        let ssri_cell = self
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

        let ssri_binary = ssri_cell
            .cell
            .ok_or(Error::InvalidRequest("Cell not found"))?
            .data
            .ok_or(Error::InvalidRequest("Cell doesn't have data"))?
            .content
            .into_bytes();

        let args = args.into_iter().map(|v| v.hex.into()).collect();
        let script = script.map(Into::into);
        let cell = cell.map(Into::into);
        let tx = tx.map(|v| v.inner.into());

        Ok(
            execute_riscv_binary(self.rpc.clone(), ssri_binary, args, script, cell, tx)?
                .map(|v| v.into()),
        )
    }
}

#[async_trait]
impl SSRILevel for SSRIRunner {
    async fn run_script_level_code(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        self.run_script(tx_hash, index, args, None, None, None)
            .await
    }

    async fn run_script_level_script(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        script: Script,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        self.run_script(tx_hash, index, args, Some(script), None, None)
            .await
    }

    async fn run_script_level_cell(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        cell: CellOutputWithData,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        self.run_script(tx_hash, index, args, None, Some(cell), None)
            .await
    }

    async fn run_script_level_tx(
        &self,
        tx_hash: H256,
        index: u32,
        args: Vec<Hex>,
        tx: TransactionView,
    ) -> Result<Option<Hex>, ErrorObjectOwned> {
        self.run_script(tx_hash, index, args, None, None, Some(tx))
            .await
    }
}
