# Changelog

## 0.2.0 - 2026-08-14

- Upgrade the Alloy integration to the 2.4.x release series.
- Extend `reth_newPayload` with Reth big-block data and raw RLP blocks with an optional merged
  block access list.
- Preserve the three-parameter `reth_newPayload` wire format and the simplified
  `reth_forkchoiceUpdated` request.
- Preserve optional Reth payload timing fields during response decoding.

### Migration

`RethNewPayloadInput::BlockRlp(Bytes)` is now
`RethNewPayloadInput::BlockRlp { block, bal }`. Use `RethNewPayloadInput::block_rlp(bytes)` for
the old bare-byte behavior, or `RethNewPayloadInput::block_rlp_with_bal(block, bal)` for a block
access list.

## 0.1.0

- Initial release of the Reth RPC types and provider extension trait.
