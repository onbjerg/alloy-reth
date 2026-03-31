//! Types for the `reth_` RPC namespace.

use alloy_eips::BlockId;
use alloy_primitives::{Address, Bytes, U64, U256, map::HashMap};
use alloy_rpc_types_engine::PayloadStatus;
use serde::{Deserialize, Serialize};

/// Response type for `reth_getBalanceChangesInBlock`.
///
/// Maps addresses to their updated balances after block execution.
pub type BalanceChangesInBlock = HashMap<Address, U256>;

/// The input to `reth_newPayload`.
///
/// Accepts either a standard execution payload (with sidecar data) or raw RLP-encoded block bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RethNewPayloadInput<ExecutionData> {
    /// Standard execution data (payload + sidecar).
    ExecutionData(ExecutionData),
    /// Raw RLP-encoded block bytes.
    BlockRlp(Bytes),
}

impl<E> RethNewPayloadInput<E> {
    /// Creates a new [`RethNewPayloadInput`] from execution data.
    pub const fn execution_data(data: E) -> Self {
        Self::ExecutionData(data)
    }

    /// Creates a new [`RethNewPayloadInput`] from raw RLP-encoded block bytes.
    pub fn block_rlp(bytes: impl Into<Bytes>) -> Self {
        Self::BlockRlp(bytes.into())
    }
}

/// Parameters for `reth_newPayload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RethNewPayloadParams<E = serde_json::Value> {
    /// The payload input.
    pub payload: RethNewPayloadInput<E>,
    /// Whether to wait for persistence before returning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_persistence: Option<bool>,
    /// Whether to wait for caches before returning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_caches: Option<bool>,
}

impl<E> RethNewPayloadParams<E> {
    /// Creates new [`RethNewPayloadParams`] with the given payload input.
    pub const fn new(payload: RethNewPayloadInput<E>) -> Self {
        Self {
            payload,
            wait_for_persistence: None,
            wait_for_caches: None,
        }
    }

    /// Sets whether to wait for persistence.
    pub const fn with_wait_for_persistence(mut self, wait: bool) -> Self {
        self.wait_for_persistence = Some(wait);
        self
    }

    /// Sets whether to wait for caches.
    pub const fn with_wait_for_caches(mut self, wait: bool) -> Self {
        self.wait_for_caches = Some(wait);
        self
    }
}

impl<E> From<RethNewPayloadInput<E>> for RethNewPayloadParams<E> {
    fn from(payload: RethNewPayloadInput<E>) -> Self {
        Self::new(payload)
    }
}

/// Extended payload status returned by `reth_newPayload`.
///
/// Wraps the standard [`PayloadStatus`] with server-side timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RethPayloadStatus {
    /// The standard payload status.
    #[serde(flatten)]
    pub status: PayloadStatus,
    /// Total execution latency in microseconds.
    pub latency_us: u64,
    /// Time spent waiting for persistence in microseconds, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_wait_us: Option<u64>,
    /// Time spent waiting for the execution cache lock in microseconds, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_cache_wait_us: Option<u64>,
    /// Time spent waiting for the sparse trie lock in microseconds, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sparse_trie_wait_us: Option<u64>,
}

impl RethPayloadStatus {
    /// Creates a new [`RethPayloadStatus`] with the given status and latency.
    pub const fn new(status: PayloadStatus, latency_us: u64) -> Self {
        Self {
            status,
            latency_us,
            persistence_wait_us: None,
            execution_cache_wait_us: None,
            sparse_trie_wait_us: None,
        }
    }

    /// Sets the persistence wait time.
    pub const fn with_persistence_wait_us(mut self, us: u64) -> Self {
        self.persistence_wait_us = Some(us);
        self
    }

    /// Sets the execution cache wait time.
    pub const fn with_execution_cache_wait_us(mut self, us: u64) -> Self {
        self.execution_cache_wait_us = Some(us);
        self
    }

    /// Sets the sparse trie wait time.
    pub const fn with_sparse_trie_wait_us(mut self, us: u64) -> Self {
        self.sparse_trie_wait_us = Some(us);
        self
    }
}

impl AsRef<PayloadStatus> for RethPayloadStatus {
    fn as_ref(&self) -> &PayloadStatus {
        &self.status
    }
}

/// Parameters for `reth_getBlockExecutionOutcome`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GetBlockExecutionOutcomeParams {
    /// The block identifier.
    pub block_id: BlockId,
    /// Optional number of blocks to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<U64>,
}

impl GetBlockExecutionOutcomeParams {
    /// Creates new [`GetBlockExecutionOutcomeParams`] for the given block.
    pub const fn new(block_id: BlockId) -> Self {
        Self {
            block_id,
            count: None,
        }
    }

    /// Sets the number of blocks to include.
    pub fn with_count(mut self, count: impl Into<U64>) -> Self {
        self.count = Some(count.into());
        self
    }
}

impl From<BlockId> for GetBlockExecutionOutcomeParams {
    fn from(block_id: BlockId) -> Self {
        Self::new(block_id)
    }
}

/// Notification emitted by `reth_subscribeChainNotifications`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum CanonStateNotification {
    /// New chain segment was committed.
    Commit {
        /// The newly committed chain segment, serialized as JSON value.
        new: serde_json::Value,
    },
    /// Chain reorganization occurred.
    Reorg {
        /// The reverted chain segment, serialized as JSON value.
        old: serde_json::Value,
        /// The replacement chain segment, serialized as JSON value.
        new: serde_json::Value,
    },
}

impl CanonStateNotification {
    /// Creates a [`Commit`](Self::Commit) notification.
    pub const fn commit(new: serde_json::Value) -> Self {
        Self::Commit { new }
    }

    /// Creates a [`Reorg`](Self::Reorg) notification.
    pub const fn reorg(old: serde_json::Value, new: serde_json::Value) -> Self {
        Self::Reorg { old, new }
    }

    /// Returns `true` if this is a [`Commit`](Self::Commit) notification.
    pub const fn is_commit(&self) -> bool {
        matches!(self, Self::Commit { .. })
    }

    /// Returns `true` if this is a [`Reorg`](Self::Reorg) notification.
    pub const fn is_reorg(&self) -> bool {
        matches!(self, Self::Reorg { .. })
    }
}
