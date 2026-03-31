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
    /// This is an extended version of `engine_newPayload` that accepts either standard execution
    /// data or raw RLP-encoded block bytes, and returns timing information.
    ///
    /// ## JSON-RPC Method
    ///
    /// `reth_newPayload`
    async fn reth_new_payload(
        &self,
        params: impl Into<RethNewPayloadParams> + Send,
    ) -> TransportResult<RethPayloadStatus>;

    /// Updates the forkchoice state.
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

    async fn reth_new_payload(
        &self,
        params: impl Into<RethNewPayloadParams> + Send,
    ) -> TransportResult<RethPayloadStatus> {
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
