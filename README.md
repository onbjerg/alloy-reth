# alloy-reth

Types and provider extensions for the `reth_` Ethereum JSON-RPC namespace.

## Usage

```toml
[dependencies]
alloy-reth = "0.1"
```

### Provider Extension

With the `provider` feature (enabled by default), you can use the `RethApi` extension trait
with any alloy provider:

```rust,ignore
use alloy_provider::ProviderBuilder;
use alloy_reth::RethApi;
use alloy_eips::BlockId;

let provider = ProviderBuilder::new().connect("http://localhost:8545").await?;

let changes = provider
    .reth_get_balance_changes_in_block(BlockId::latest())
    .await?;
```

## Supported Methods

| Method | Description |
|--------|-------------|
| `reth_getBalanceChangesInBlock` | Returns balance changes in a block |
| `reth_getBlockExecutionOutcome` | Returns the execution outcome for a block |
| `reth_newPayload` | Submits a new payload to the engine (extended) |
| `reth_forkchoiceUpdated` | Updates the forkchoice state |

## Features

- `std` — Enable `std` support (enabled by default)
- `provider` — Enable the `RethApi` provider extension trait (enabled by default)

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
