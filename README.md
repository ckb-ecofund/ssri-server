# SSRI Server

Start server with

```sh
RUST_LOG=ssri_server=debug cargo run
```

Run a script with

```sh
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "run_script",
    "params": ["0x08a3a1063b2cdd727ce5fc232448f816be97a64b189ec969b163841ac37a4aae", 0, ["0xaa", "0x1234"]]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8090
```