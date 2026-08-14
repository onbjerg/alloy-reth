//! Provider extension trait for the `reth_` RPC namespace.

use crate::types::{
    BalanceChangesInBlock, GetBlockExecutionOutcomeParams, RethNewPayloadParams, RethPayloadStatus,
};
use alloy_network::Network;
use alloy_provider::Provider;
use alloy_rpc_types_engine::{ForkchoiceState, ForkchoiceUpdated};
use alloy_transport::TransportResult;

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}

/// Extension trait for the `reth_` RPC namespace.
///
/// Provides access to reth-specific RPC methods through the alloy provider. This trait is
/// sealed and cannot be implemented outside of this crate.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait RethApi<N: Network>: sealed::Sealed + Send + Sync {
    /// Returns the balance changes in the given block.
    ///
    /// ## JSON-RPC Method
    ///
    /// `reth_getBalanceChangesInBlock`
    async fn reth_get_balance_changes_in_block(
        &self,
        params: impl Into<GetBlockExecutionOutcomeParams> + Send,
    ) -> TransportResult<BalanceChangesInBlock>;

    /// Returns the execution outcome for the given block.
    ///
    /// ## JSON-RPC Method
    ///
    /// `reth_getBlockExecutionOutcome`
    async fn reth_get_block_execution_outcome(
        &self,
        params: impl Into<GetBlockExecutionOutcomeParams> + Send,
    ) -> TransportResult<Option<serde_json::Value>>;

    /// Submits a new payload to the engine.
    ///
    /// This is an extended version of `engine_newPayload` that accepts standard execution data,
    /// big-block data, or raw RLP-encoded block bytes, and returns timing information.
    ///
    /// The request always contains three positional parameters: the payload, the optional
    /// persistence wait flag, and the optional cache wait flag.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use alloy_reth::{RethApi, RethNewPayloadInput, RethNewPayloadParams};
    ///
    /// let payload = RethNewPayloadInput::execution_data(serde_json::json!({
    ///     "payload": "0x01",
    ///     "sidecar": "0x02",
    /// }));
    /// provider
    ///     .reth_new_payload(RethNewPayloadParams::new(payload))
    ///     .await?;
    /// # Ok::<(), alloy_transport::TransportError>(())
    /// ```
    ///
    /// ## JSON-RPC Method
    ///
    /// `reth_newPayload`
    async fn reth_new_payload<E>(
        &self,
        params: impl Into<RethNewPayloadParams<E>> + Send,
    ) -> TransportResult<RethPayloadStatus>
    where
        E: serde::Serialize + Clone + core::fmt::Debug + Send + Sync + Unpin + 'static;

    /// Updates the forkchoice state.
    ///
    /// This Reth-specific endpoint intentionally sends only the forkchoice state. It does not
    /// accept or send payload attributes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use alloy_primitives::B256;
    /// use alloy_reth::RethApi;
    /// use alloy_rpc_types_engine::ForkchoiceState;
    ///
    /// provider
    ///     .reth_forkchoice_updated(ForkchoiceState {
    ///         head_block_hash: B256::ZERO,
    ///         safe_block_hash: B256::ZERO,
    ///         finalized_block_hash: B256::ZERO,
    ///     })
    ///     .await?;
    /// # Ok::<(), alloy_transport::TransportError>(())
    /// ```
    ///
    /// ## JSON-RPC Method
    ///
    /// `reth_forkchoiceUpdated`
    async fn reth_forkchoice_updated(
        &self,
        forkchoice_state: ForkchoiceState,
    ) -> TransportResult<ForkchoiceUpdated>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<N, P> RethApi<N> for P
where
    N: Network,
    P: Provider<N>,
{
    async fn reth_get_balance_changes_in_block(
        &self,
        params: impl Into<GetBlockExecutionOutcomeParams> + Send,
    ) -> TransportResult<BalanceChangesInBlock> {
        let params = params.into();
        self.client()
            .request("reth_getBalanceChangesInBlock", (params.block_id,))
            .await
    }

    async fn reth_get_block_execution_outcome(
        &self,
        params: impl Into<GetBlockExecutionOutcomeParams> + Send,
    ) -> TransportResult<Option<serde_json::Value>> {
        let params = params.into();
        self.client()
            .request(
                "reth_getBlockExecutionOutcome",
                (params.block_id, params.count),
            )
            .await
    }

    async fn reth_new_payload<E>(
        &self,
        params: impl Into<RethNewPayloadParams<E>> + Send,
    ) -> TransportResult<RethPayloadStatus>
    where
        E: serde::Serialize + Clone + core::fmt::Debug + Send + Sync + Unpin + 'static,
    {
        let params = params.into();
        self.client()
            .request(
                "reth_newPayload",
                (
                    params.payload,
                    params.wait_for_persistence,
                    params.wait_for_caches,
                ),
            )
            .await
    }

    async fn reth_forkchoice_updated(
        &self,
        forkchoice_state: ForkchoiceState,
    ) -> TransportResult<ForkchoiceUpdated> {
        self.client()
            .request("reth_forkchoiceUpdated", (forkchoice_state,))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BigBlockData, RethNewPayloadInput};
    use alloy_json_rpc::RequestPacket;
    use alloy_network::Ethereum;
    use alloy_primitives::{Address, B256, Bytes, U64, U256};
    use alloy_provider::{Provider, ProviderBuilder};
    use alloy_rpc_client::RpcClient;
    use alloy_rpc_types_engine::{ForkchoiceState, PayloadStatusEnum};
    use alloy_transport::{TransportError, TransportFut, mock::Asserter};
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};
    use tower::Service;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestExecutionData {
        payload: Bytes,
        sidecar: Bytes,
    }

    #[derive(Clone, Debug)]
    struct RecordingTransport {
        inner: alloy_transport::mock::MockTransport,
        requests: Arc<Mutex<Vec<RequestPacket>>>,
    }

    impl Service<RequestPacket> for RecordingTransport {
        type Response = alloy_json_rpc::ResponsePacket;
        type Error = TransportError;
        type Future = TransportFut<'static>;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: RequestPacket) -> Self::Future {
            self.requests.lock().unwrap().push(request.clone());
            self.inner.call(request)
        }
    }

    fn provider_with_asserter(
        asserter: Asserter,
    ) -> (impl Provider<Ethereum>, Arc<Mutex<Vec<RequestPacket>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = RecordingTransport {
            inner: alloy_transport::mock::MockTransport::new(asserter),
            requests: Arc::clone(&requests),
        };
        let provider = ProviderBuilder::new().connect_client(RpcClient::new(transport, true));
        (provider, requests)
    }

    fn take_request(requests: &Arc<Mutex<Vec<RequestPacket>>>) -> Value {
        let packet = requests.lock().unwrap().remove(0);
        let request = packet
            .as_single()
            .expect("expected a single JSON-RPC request");
        serde_json::from_str(request.serialized().get()).unwrap()
    }

    fn assert_request(requests: &Arc<Mutex<Vec<RequestPacket>>>, method: &str, params: Value) {
        let request = take_request(requests);
        assert_eq!(request["method"], method);
        assert_eq!(request["params"], params);
    }

    fn push_valid_status(asserter: &Asserter, with_timings: bool) {
        let response = if with_timings {
            json!({
                "status": "VALID",
                "latestValidHash": null,
                "latency_us": 12,
                "persistence_wait_us": 3,
                "execution_cache_wait_us": 4,
                "sparse_trie_wait_us": 5,
            })
        } else {
            json!({"status": "VALID", "latestValidHash": null})
        };
        asserter.push_success(&response);
    }

    fn execution_data() -> TestExecutionData {
        TestExecutionData {
            payload: Bytes::from_static(&[0x01]),
            sidecar: Bytes::from_static(&[0x02]),
        }
    }

    #[tokio::test]
    async fn balance_changes_uses_exact_reth_method_and_params() {
        let asserter = Asserter::new();
        asserter.push_success(&json!({
            "0x1111111111111111111111111111111111111111": "0x02"
        }));
        let (provider, requests) = provider_with_asserter(asserter);

        let changes = provider
            .reth_get_balance_changes_in_block(alloy_eips::BlockId::latest())
            .await
            .unwrap();

        assert_eq!(
            changes.get(&Address::from([0x11; 20])),
            Some(&U256::from(2))
        );
        assert_request(
            &requests,
            "reth_getBalanceChangesInBlock",
            json!(["latest"]),
        );
    }

    #[tokio::test]
    async fn execution_outcome_uses_exact_reth_method_and_params() {
        let asserter = Asserter::new();
        asserter.push_success(&json!({"state": "0x01"}));
        let (provider, requests) = provider_with_asserter(asserter);

        let outcome = provider
            .reth_get_block_execution_outcome(
                GetBlockExecutionOutcomeParams::new(alloy_eips::BlockId::latest())
                    .with_count(U64::from_limbs([2])),
            )
            .await
            .unwrap();

        assert_eq!(outcome, Some(json!({"state": "0x01"})));
        assert_request(
            &requests,
            "reth_getBlockExecutionOutcome",
            json!(["latest", "0x2"]),
        );
    }

    #[tokio::test]
    async fn new_payload_sends_all_variants_and_both_wait_flags() {
        let asserter = Asserter::new();
        push_valid_status(&asserter, true);
        push_valid_status(&asserter, false);
        push_valid_status(&asserter, false);
        let (provider, requests) = provider_with_asserter(asserter);

        let standard = RethNewPayloadInput::execution_data(execution_data());
        let status = provider
            .reth_new_payload(
                RethNewPayloadParams::new(standard)
                    .with_wait_for_persistence(true)
                    .with_wait_for_caches(false),
            )
            .await
            .unwrap();
        assert_eq!(status.status.status, PayloadStatusEnum::Valid);
        assert_eq!(status.latency_us, 12);
        assert_eq!(status.persistence_wait_us, Some(3));
        assert_eq!(status.execution_cache_wait_us, Some(4));
        assert_eq!(status.sparse_trie_wait_us, Some(5));
        assert_request(
            &requests,
            "reth_newPayload",
            json!([{"payload": "0x01", "sidecar": "0x02"}, true, false]),
        );

        let hash = B256::from([0x11; 32]);
        let big_block = RethNewPayloadInput::big_block_data(BigBlockData {
            env_switches: vec![execution_data()],
            prior_block_hashes: vec![(7, hash)],
            block_number: 8,
            merged_block_access_list: None,
        });
        let status = provider.reth_new_payload(big_block).await.unwrap();
        assert_eq!(status.persistence_wait_us, None);
        assert_eq!(status.execution_cache_wait_us, None);
        assert_eq!(status.sparse_trie_wait_us, None);
        let hash = serde_json::to_value(hash).unwrap();
        assert_request(
            &requests,
            "reth_newPayload",
            json!([{
                "env_switches": [{"payload": "0x01", "sidecar": "0x02"}],
                "prior_block_hashes": [[7, hash]],
                "block_number": 8,
            }, null, null]),
        );

        let raw = RethNewPayloadInput::<serde_json::Value>::block_rlp_with_bal(
            Bytes::from_static(&[0x04]),
            Bytes::from_static(&[0x05]),
        );
        provider
            .reth_new_payload(RethNewPayloadParams::new(raw).with_wait_for_caches(true))
            .await
            .unwrap();
        assert_request(
            &requests,
            "reth_newPayload",
            json!([{"block": "0x04", "bal": "0x05"}, null, true]),
        );
    }

    #[tokio::test]
    async fn forkchoice_updated_sends_only_state_without_payload_attributes() {
        let asserter = Asserter::new();
        asserter.push_success(&json!({
            "payloadStatus": {"status": "VALID", "latestValidHash": null},
            "payloadId": null,
        }));
        let (provider, requests) = provider_with_asserter(asserter);
        let state = ForkchoiceState::same_hash(B256::from([0x11; 32]));

        let response = provider.reth_forkchoice_updated(state).await.unwrap();
        assert!(response.payload_status.is_valid());

        assert_request(
            &requests,
            "reth_forkchoiceUpdated",
            json!([serde_json::to_value(state).unwrap()]),
        );
    }
}
