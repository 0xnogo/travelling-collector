# Travelling Collector

## How to use

Set the following environment variables:
```bash
RPC_WS_ENDPOINT=...
export RPC_WS_ENDPOINT
```

Build
```bash
cargo build
```

Run the script
```bash
cargo run -- $MODE $PATH_TO_FILE $STARTING_BLOCK
```

2 possible modes:
* `backwards`: start from the `$STARTING_BLOCK` and will go backwards (`$STARTING_BLOCK` should be set)
* `scanning`: listen to new created blocks and will go forward (`$STARTING_BLOCK` should not be set)
* `path_to_file`: path to the file where the reporting will be stored (append if file already exists)