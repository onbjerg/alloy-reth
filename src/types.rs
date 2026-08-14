//! Types for the `reth_` RPC namespace.

use alloy_eips::BlockId;
use alloy_primitives::{Address, B256, Bytes, U64, U256, map::HashMap};
use alloy_rpc_types_engine::PayloadStatus;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};

/// Response type for `reth_getBalanceChangesInBlock`.
///
/// Maps addresses to their updated balances after block execution.
pub type BalanceChangesInBlock = HashMap<Address, U256>;

/// The input to `reth_newPayload`.
///
/// Reth accepts standard execution data, big-block execution data, and raw RLP-encoded blocks.
/// Raw blocks are sent as a legacy bare byte string when no block access list is present, and as
/// an object containing `block` and `bal` when one is present.
///
/// # Examples
///
/// Standard execution data can use any serializable execution-data type:
///
/// ```
/// use alloy_reth::RethNewPayloadInput;
///
/// let input = RethNewPayloadInput::execution_data(serde_json::json!({
///     "payload": "0x01",
///     "sidecar": "0x02",
/// }));
/// assert!(serde_json::to_value(input).is_ok());
/// ```
///
/// Big-block data carries the constituent execution environments and their block hashes:
///
/// ```
/// use alloy_primitives::B256;
/// use alloy_reth::{BigBlockData, RethNewPayloadInput};
///
/// let input = RethNewPayloadInput::big_block_data(BigBlockData {
///     env_switches: vec![serde_json::json!({"payload": "0x01"})],
///     prior_block_hashes: vec![(7, B256::ZERO)],
///     block_number: 8,
///     merged_block_access_list: None,
/// });
/// assert!(serde_json::to_value(input).is_ok());
/// ```
///
/// Raw RLP data may be sent with an optional merged block access list:
///
/// ```
/// use alloy_primitives::Bytes;
/// use alloy_reth::RethNewPayloadInput;
///
/// let input = RethNewPayloadInput::<serde_json::Value>::block_rlp_with_bal(
///     Bytes::from_static(&[0x01]),
///     Bytes::from_static(&[0x02]),
/// );
/// assert_eq!(serde_json::to_value(input).unwrap()["block"], "0x01");
/// ```
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum RethNewPayloadInput<ExecutionData> {
    /// Standard execution data (payload + sidecar).
    ExecutionData(ExecutionData),
    /// Big-block execution data.
    BigBlockData(Box<BigBlockData<ExecutionData>>),
    /// Raw RLP-encoded block bytes and an optional merged block access list.
    BlockRlp {
        /// RLP-encoded block bytes.
        block: Bytes,
        /// Optional merged block access list bytes.
        bal: Option<Bytes>,
    },
}

impl<E> Serialize for RethNewPayloadInput<E>
where
    E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::ExecutionData(data) => data.serialize(serializer),
            Self::BigBlockData(data) => data.serialize(serializer),
            Self::BlockRlp { block, bal: None } => block.serialize(serializer),
            Self::BlockRlp {
                block,
                bal: Some(bal),
            } => {
                let mut object = serializer.serialize_struct("RethNewPayloadBlockRlp", 2)?;
                object.serialize_field("block", block)?;
                object.serialize_field("bal", bal)?;
                object.end()
            }
        }
    }
}

impl<'de, E> Deserialize<'de> for RethNewPayloadInput<E>
where
    E: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr<E> {
            BlockRlp {
                block: Bytes,
                #[serde(default)]
                bal: Option<Bytes>,
            },
            LegacyBlockRlp(Bytes),
            BigBlockData(Box<BigBlockData<E>>),
            ExecutionData(E),
        }

        Ok(match Repr::deserialize(deserializer)? {
            Repr::BlockRlp { block, bal } => Self::BlockRlp { block, bal },
            Repr::LegacyBlockRlp(block) => Self::BlockRlp { block, bal: None },
            Repr::BigBlockData(data) => Self::BigBlockData(data),
            Repr::ExecutionData(data) => Self::ExecutionData(data),
        })
    }
}

/// Big-block execution data accepted by `reth_newPayload`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BigBlockData<ExecutionData> {
    /// Execution data for each constituent environment in the big block.
    pub env_switches: Vec<ExecutionData>,
    /// Block numbers and hashes preceding the big block.
    pub prior_block_hashes: Vec<(u64, B256)>,
    /// Number of the big block.
    pub block_number: u64,
    /// Optional merged block access list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_block_access_list: Option<Bytes>,
}

/// Compatibility alias for [`BigBlockData`].
pub type RethBigBlockData<ExecutionData> = BigBlockData<ExecutionData>;

impl<ExecutionData> BigBlockData<ExecutionData> {
    /// Creates big-block data without a merged block access list.
    pub const fn new(
        env_switches: Vec<ExecutionData>,
        prior_block_hashes: Vec<(u64, B256)>,
        block_number: u64,
    ) -> Self {
        Self {
            env_switches,
            prior_block_hashes,
            block_number,
            merged_block_access_list: None,
        }
    }

    /// Sets the merged block access list.
    pub fn with_merged_block_access_list(mut self, bal: impl Into<Bytes>) -> Self {
        self.merged_block_access_list = Some(bal.into());
        self
    }
}

impl<E> RethNewPayloadInput<E> {
    /// Creates a new [`RethNewPayloadInput`] from execution data.
    pub const fn execution_data(data: E) -> Self {
        Self::ExecutionData(data)
    }

    /// Creates a new [`RethNewPayloadInput`] from big-block execution data.
    pub fn big_block_data(data: BigBlockData<E>) -> Self {
        Self::BigBlockData(Box::new(data))
    }

    /// Creates a new [`RethNewPayloadInput`] from raw RLP-encoded block bytes.
    pub fn block_rlp(bytes: impl Into<Bytes>) -> Self {
        Self::BlockRlp {
            block: bytes.into(),
            bal: None,
        }
    }

    /// Creates a new [`RethNewPayloadInput`] from raw RLP bytes and a block access list.
    pub fn block_rlp_with_bal(block: impl Into<Bytes>, bal: impl Into<Bytes>) -> Self {
        Self::BlockRlp {
            block: block.into(),
            bal: Some(bal.into()),
        }
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
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_rpc_types_engine::PayloadStatusEnum;
    use serde_json::json;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestExecutionData {
        payload: Bytes,
        sidecar: Bytes,
    }

    fn execution_data() -> TestExecutionData {
        TestExecutionData {
            payload: Bytes::from_static(&[0x01]),
            sidecar: Bytes::from_static(&[0x02]),
        }
    }

    #[test]
    fn execution_data_round_trips() {
        let input = RethNewPayloadInput::execution_data(execution_data());
        let value = serde_json::to_value(&input).unwrap();
        assert_eq!(value, json!({"payload": "0x01", "sidecar": "0x02"}));

        let decoded: RethNewPayloadInput<TestExecutionData> =
            serde_json::from_value(value).unwrap();
        assert!(
            matches!(decoded, RethNewPayloadInput::ExecutionData(data) if data == execution_data())
        );
    }

    #[test]
    fn big_block_data_round_trips_with_and_without_bal() {
        let hash = B256::from([0x11; 32]);
        let data = BigBlockData {
            env_switches: vec![execution_data()],
            prior_block_hashes: vec![(7, hash)],
            block_number: 8,
            merged_block_access_list: None,
        };
        let input = RethNewPayloadInput::big_block_data(data.clone());
        let value = serde_json::to_value(&input).unwrap();
        assert_eq!(
            value,
            json!({
                "env_switches": [{"payload": "0x01", "sidecar": "0x02"}],
                "prior_block_hashes": [[7, hash]],
                "block_number": 8,
            })
        );

        let decoded: RethNewPayloadInput<TestExecutionData> =
            serde_json::from_value(value).unwrap();
        assert!(matches!(decoded, RethNewPayloadInput::BigBlockData(decoded) if *decoded == data));

        let with_bal = data.with_merged_block_access_list(Bytes::from_static(&[0x03]));
        let input = RethNewPayloadInput::big_block_data(with_bal.clone());
        let value = serde_json::to_value(&input).unwrap();
        assert_eq!(value["merged_block_access_list"], "0x03");
        let decoded: RethNewPayloadInput<TestExecutionData> =
            serde_json::from_value(value).unwrap();
        assert!(
            matches!(decoded, RethNewPayloadInput::BigBlockData(decoded) if *decoded == with_bal)
        );
    }

    #[test]
    fn raw_rlp_serializes_as_legacy_bytes_without_bal() {
        let input =
            RethNewPayloadInput::<TestExecutionData>::block_rlp(Bytes::from_static(&[0x01, 0x02]));
        assert_eq!(serde_json::to_value(&input).unwrap(), json!("0x0102"));

        let decoded: RethNewPayloadInput<TestExecutionData> =
            serde_json::from_value(json!("0x0102")).unwrap();
        assert!(
            matches!(decoded, RethNewPayloadInput::BlockRlp { block, bal: None } if block == Bytes::from_static(&[0x01, 0x02]))
        );
    }

    #[test]
    fn raw_rlp_serializes_as_object_with_bal_and_decodes_legacy_object() {
        let input = RethNewPayloadInput::<TestExecutionData>::block_rlp_with_bal(
            Bytes::from_static(&[0x01]),
            Bytes::from_static(&[0x02]),
        );
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            json!({"block": "0x01", "bal": "0x02"})
        );

        for value in [
            json!({"block": "0x01", "bal": "0x02"}),
            json!({"block": "0x01"}),
        ] {
            let decoded: RethNewPayloadInput<TestExecutionData> =
                serde_json::from_value(value).unwrap();
            assert!(
                matches!(decoded, RethNewPayloadInput::BlockRlp { block, .. } if block == Bytes::from_static(&[0x01]))
            );
        }
    }

    #[test]
    fn new_payload_params_omit_unset_wait_flags() {
        let params =
            RethNewPayloadParams::new(RethNewPayloadInput::execution_data(execution_data()));
        assert_eq!(
            serde_json::to_value(params).unwrap(),
            json!({"payload": {"payload": "0x01", "sidecar": "0x02"}})
        );

        let params =
            RethNewPayloadParams::new(RethNewPayloadInput::execution_data(execution_data()))
                .with_wait_for_persistence(true)
                .with_wait_for_caches(false);
        assert_eq!(
            serde_json::to_value(params).unwrap(),
            json!({
                "payload": {"payload": "0x01", "sidecar": "0x02"},
                "wait_for_persistence": true,
                "wait_for_caches": false,
            })
        );
    }

    #[test]
    fn payload_status_preserves_all_timing_fields() {
        let status: RethPayloadStatus = serde_json::from_value(json!({
            "status": "VALID",
            "latestValidHash": null,
            "latency_us": 12,
            "persistence_wait_us": 3,
            "execution_cache_wait_us": 4,
            "sparse_trie_wait_us": 5,
        }))
        .unwrap();

        assert_eq!(status.status.status, PayloadStatusEnum::Valid);
        assert_eq!(status.latency_us, 12);
        assert_eq!(status.persistence_wait_us, Some(3));
        assert_eq!(status.execution_cache_wait_us, Some(4));
        assert_eq!(status.sparse_trie_wait_us, Some(5));
    }

    #[test]
    fn payload_status_accepts_missing_timing_fields() {
        let status: RethPayloadStatus = serde_json::from_value(json!({
            "status": "VALID",
            "latestValidHash": null,
        }))
        .unwrap();

        assert_eq!(status.status.status, PayloadStatusEnum::Valid);
        assert_eq!(status.latency_us, 0);
        assert_eq!(status.persistence_wait_us, None);
        assert_eq!(status.execution_cache_wait_us, None);
        assert_eq!(status.sparse_trie_wait_us, None);
    }
}
