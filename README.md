# SSRI Runner

This repo will compile to a binary that can run SSRI scripts and also start a RPC server to run scripts remotely.

Run a script with

```sh
RUST_LOG=ssri_cli=debug cargo run -- run --tx-hash 0x900afcf79235e88f7bdf8a5d320365b7912f8074f4489a68405f43586fc51e5c --index 0 0x58f02409de9de7b1 0x0000000000000000 0x0a00000000000000
```

Start server with

```sh
RUST_LOG=ssri_cli=debug cargo run -- server --ckb-rpc https://testnet.ckbapp.dev/ --server-addr localhost:9090
```

Send request to server with

```sh
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "run_script_level_code",
    "params": ["0x900afcf79235e88f7bdf8a5d320365b7912f8074f4489a68405f43586fc51e5c", 0, ["0x58f02409de9de7b1", "0x0000000000000000", "0x0a00000000000000"]]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:9090
```
