#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Types for the `reth_` RPC namespace.
pub mod types;
pub use types::{
    BalanceChangesInBlock, CanonStateNotification, GetBlockExecutionOutcomeParams,
    RethNewPayloadInput, RethNewPayloadParams, RethPayloadStatus,
};

/// Provider extension for the `reth_` RPC namespace.
#[cfg(feature = "provider")]
pub mod provider;
#[cfg(feature = "provider")]
pub use provider::RethApi;
