# alloy-reth

Types and provider extensions for the `reth_` Ethereum JSON-RPC namespace.

## Usage

```toml
[dependencies]
alloy-reth = "0.2"
```

Version 0.2 targets the Alloy 2.4.x release series (with Alloy primitives 1.6.x).

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

`reth_newPayload` accepts standard execution data, Reth big-block data, and raw RLP blocks:

```rust,ignore
use alloy_primitives::{B256, Bytes};
use alloy_reth::{BigBlockData, RethApi, RethNewPayloadInput, RethNewPayloadParams};

let standard = RethNewPayloadInput::execution_data(serde_json::json!({
    "payload": "0x01",
    "sidecar": "0x02",
}));
let big_block = RethNewPayloadInput::big_block_data(BigBlockData {
    env_switches: vec![serde_json::json!({"payload": "0x01"})],
    prior_block_hashes: vec![(7, B256::ZERO)],
    block_number: 8,
    merged_block_access_list: None,
});
let raw_block = RethNewPayloadInput::block_rlp(Bytes::from_static(&[0x01]));

provider
    .reth_new_payload(
        RethNewPayloadParams::new(standard)
            .with_wait_for_persistence(true)
            .with_wait_for_caches(false),
    )
    .await?;
```

Raw RLP blocks with a merged block access list use
`RethNewPayloadInput::block_rlp_with_bal(block, bal)`. Without a BAL, the wire format remains the
legacy bare byte string; with one, it is `{ "block": ..., "bal": ... }`.

`reth_forkchoiceUpdated` sends only the forkchoice state. Payload attributes are intentionally not
part of this Reth-specific method.

### Migration from 0.1

The `BlockRlp(Bytes)` enum variant is now `BlockRlp { block, bal }` so a BAL can be represented.
Use `RethNewPayloadInput::block_rlp(bytes)` to migrate old code, or
`RethNewPayloadInput::block_rlp_with_bal(block, bal)` when supplying a BAL. The `block_rlp`
constructor also preserves legacy bare-byte serialization.

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
