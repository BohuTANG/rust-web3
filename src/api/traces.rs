use crate::{
    api::Namespace,
    helpers::{self, CallFuture},
    types::{BlockId, BlockNumber, BlockTrace, Bytes, CallRequest, Index, Trace, TraceFilter, TraceType, H256},
    Transport,
};
use std::collections::HashMap;

/// `Trace` namespace
#[derive(Debug, Clone)]
pub struct Traces<T> {
    transport: T,
}

impl<T: Transport> Namespace<T> for Traces<T> {
    fn new(transport: T) -> Self
    where
        Self: Sized,
    {
        Traces { transport }
    }

    fn transport(&self) -> &T {
        &self.transport
    }
}

impl<T: Transport> Traces<T> {
    /// Executes the given call and returns a number of possible traces for it
    pub fn call(
        &self,
        req: CallRequest,
        trace_type: Vec<TraceType>,
        block: Option<BlockNumber>,
    ) -> CallFuture<BlockTrace, T::Out> {
        let req = helpers::serialize(&req);
        let block = helpers::serialize(&block.unwrap_or(BlockNumber::Latest));
        let trace_type = helpers::serialize(&trace_type);
        CallFuture::new(self.transport.execute("trace_call", vec![req, trace_type, block]))
    }

    /// Performs multiple call traces on top of the same block. Allows to trace dependent transactions.
    pub fn call_many(
        &self,
        reqs_with_trace_types: Vec<(CallRequest, Vec<TraceType>)>,
        block: Option<BlockId>,
    ) -> CallFuture<Vec<BlockTrace>, T::Out> {
        let reqs_with_trace_types = helpers::serialize(&reqs_with_trace_types);
        let block = helpers::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
        CallFuture::new(
            self.transport
                .execute("trace_callMany", vec![reqs_with_trace_types, block]),
        )
    }

    /// Traces a call to `eth_sendRawTransaction` without making the call, returning the traces
    pub fn raw_transaction(&self, data: Bytes, trace_type: Vec<TraceType>) -> CallFuture<BlockTrace, T::Out> {
        let data = helpers::serialize(&data);
        let trace_type = helpers::serialize(&trace_type);
        CallFuture::new(self.transport.execute("trace_rawTransaction", vec![data, trace_type]))
    }

    /// Replays a transaction, returning the traces
    pub fn replay_transaction(&self, hash: H256, trace_type: Vec<TraceType>) -> CallFuture<BlockTrace, T::Out> {
        let hash = helpers::serialize(&hash);
        let trace_type = helpers::serialize(&trace_type);
        CallFuture::new(
            self.transport
                .execute("trace_replayTransaction", vec![hash, trace_type]),
        )
    }

    /// Replays all transactions in a block returning the requested traces for each transaction
    pub fn replay_block_transactions(
        &self,
        block: BlockNumber,
        trace_type: Vec<TraceType>,
    ) -> CallFuture<Vec<BlockTrace>, T::Out> {
        let block = helpers::serialize(&block);
        let trace_type = helpers::serialize(&trace_type);
        CallFuture::new(
            self.transport
                .execute("trace_replayBlockTransactions", vec![block, trace_type]),
        )
    }

    /// Returns traces created at given block
    pub fn block(&self, block: BlockNumber) -> CallFuture<Vec<Trace>, T::Out> {
        let block = helpers::serialize(&block);
        let mut map = HashMap::new();
        map.insert("tracer".to_string(), "callTracer".to_string());
        let tracer = helpers::serialize(&map);
        CallFuture::new(self.transport.execute("debug_traceBlockByNumber", vec![block, tracer]))
    }

    /// Return traces matching the given filter
    ///
    /// See [TraceFilterBuilder](../types/struct.TraceFilterBuilder.html)
    pub fn filter(&self, filter: TraceFilter) -> CallFuture<Vec<Trace>, T::Out> {
        let filter = helpers::serialize(&filter);
        CallFuture::new(self.transport.execute("trace_filter", vec![filter]))
    }

    /// Returns trace at the given position
    pub fn get(&self, hash: H256, index: Vec<Index>) -> CallFuture<Trace, T::Out> {
        let hash = helpers::serialize(&hash);
        let index = helpers::serialize(&index);
        CallFuture::new(self.transport.execute("trace_get", vec![hash, index]))
    }

    /// Returns all traces of a given transaction
    pub fn transaction(&self, hash: H256) -> CallFuture<Vec<Trace>, T::Out> {
        let hash = helpers::serialize(&hash);
        CallFuture::new(self.transport.execute("trace_transaction", vec![hash]))
    }
}

#[cfg(test)]
mod tests {
    use super::Traces;
    use crate::{
        api::Namespace,
        types::{Address, BlockNumber, BlockTrace, CallRequest, Trace, TraceFilterBuilder, TraceType, H256},
    };
    use hex_literal::hex;

    const EXAMPLE_BLOCKTRACE: &str = r#"
    {
        "output": "0x010203",
        "stateDiff": null,
        "trace": [
            {
                "action": {
                    "callType": "call",
                    "from": "0x0000000000000000000000000000000000000000",
                    "gas": "0x1dcd12f8",
                    "input": "0x",
                    "to": "0x0000000000000000000000000000000000000123",
                    "value": "0x1"
                },
                "result": {
                    "gasUsed": "0x0",
                    "output": "0x"
                },
                "subtraces": 0,
                "traceAddress": [],
                "type": "call"
            }
        ],
        "vmTrace": null
    }
    "#;

    const EXAMPLE_BLOCKTRACES: &str = r#"
	[{
        "output": "0x",
        "stateDiff": null,
        "trace": [
            {
                "action": {
                    "callType": "call",
                    "from": "0xa1e4380a3b1f749673e270229993ee55f35663b4",
                    "gas": "0x0",
                    "input": "0x",
                    "to": "0x5df9b87991262f6ba471f09758cde1c0fc1de734",
                    "value": "0x7a69"
                },
                "result": {
                    "gasUsed": "0x0",
                    "output": "0x"
                },
                "subtraces": 0,
                "traceAddress": [],
                "type": "call"
            }
        ],
        "transactionHash": "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060",
        "vmTrace": null
    }]
	"#;

    const EXAMPLE_TRACE_ARR: &str = r#"
    [
        {
            "action": {
                "callType": "call",
                "from": "0xaa7b131dc60b80d3cf5e59b5a21a666aa039c951",
                "gas": "0x0",
                "input": "0x",
                "to": "0xd40aba8166a212d6892125f079c33e6f5ca19814",
                "value": "0x4768d7effc3fbe"
            },
            "blockHash": "0x7eb25504e4c202cf3d62fd585d3e238f592c780cca82dacb2ed3cb5b38883add",
            "blockNumber": 3068185,
            "result": {
                "gasUsed": "0x0",
                "output": "0x"
            },
            "subtraces": 0,
            "traceAddress": [],
            "transactionHash": "0x07da28d752aba3b9dd7060005e554719c6205c8a3aea358599fc9b245c52f1f6",
            "transactionPosition": 0,
            "type": "call"
        }
    ]
    "#;

    const EXAMPLE_TRACE: &str = r#"
      {
          "action": {
              "callType": "call",
              "from": "0xaa7b131dc60b80d3cf5e59b5a21a666aa039c951",
              "gas": "0x0",
              "input": "0x",
              "to": "0xd40aba8166a212d6892125f079c33e6f5ca19814",
              "value": "0x4768d7effc3fbe"
          },
          "blockHash": "0x7eb25504e4c202cf3d62fd585d3e238f592c780cca82dacb2ed3cb5b38883add",
          "blockNumber": 3068185,
          "result": {
              "gasUsed": "0x0",
              "output": "0x"
          },
          "subtraces": 0,
          "traceAddress": [],
          "transactionHash": "0x07da28d752aba3b9dd7060005e554719c6205c8a3aea358599fc9b245c52f1f6",
          "transactionPosition": 0,
          "type": "call"
      }
    "#;

    rpc_test!(
    Traces:call, CallRequest {
    from: None, to: Some(Address::from_low_u64_be(0x123)),
    gas: None, gas_price: None,
    value: Some(0x1.into()), data: None,
    transaction_type: None, access_list: None,
    max_fee_per_gas: None, max_priority_fee_per_gas: None,
    }, vec![TraceType::Trace], None
    =>
    "trace_call", vec![r#"{"to":"0x0000000000000000000000000000000000000123","value":"0x1"}"#, r#"["trace"]"#, r#""latest""#];
    ::serde_json::from_str(EXAMPLE_BLOCKTRACE).unwrap()
    => ::serde_json::from_str::<BlockTrace>(EXAMPLE_BLOCKTRACE).unwrap()
    );

    rpc_test!(
    Traces:raw_transaction, hex!("01020304"), vec![TraceType::Trace]
    =>
    "trace_rawTransaction", vec![r#""0x01020304""#, r#"["trace"]"#];
    ::serde_json::from_str(EXAMPLE_BLOCKTRACE).unwrap()
    => ::serde_json::from_str::<BlockTrace>(EXAMPLE_BLOCKTRACE).unwrap()
    );

    rpc_test!(
    Traces:replay_transaction, "0000000000000000000000000000000000000000000000000000000000000123".parse::<H256>().unwrap(), vec![TraceType::Trace]
    =>
    "trace_replayTransaction", vec![r#""0x0000000000000000000000000000000000000000000000000000000000000123""#,r#"["trace"]"#];
    ::serde_json::from_str(EXAMPLE_BLOCKTRACE).unwrap()
    => ::serde_json::from_str::<BlockTrace>(EXAMPLE_BLOCKTRACE).unwrap()
    );

    rpc_test!(
    Traces:replay_block_transactions, BlockNumber::Latest, vec![TraceType::Trace]
    =>
    "trace_replayBlockTransactions", vec![r#""latest""#, r#"["trace"]"#];
    ::serde_json::from_str(EXAMPLE_BLOCKTRACES).unwrap()
    => ::serde_json::from_str::<Vec<BlockTrace>>(EXAMPLE_BLOCKTRACES).unwrap()
    );

    rpc_test!(
    Traces:block, BlockNumber::Latest
    =>
    "trace_block", vec![r#""latest""#];
    ::serde_json::from_str(EXAMPLE_TRACE_ARR).unwrap()
    => ::serde_json::from_str::<Vec<Trace>>(EXAMPLE_TRACE_ARR).unwrap()
    );

    rpc_test!(
    Traces:filter, TraceFilterBuilder::default().build() => "trace_filter", vec!["{}"];
    ::serde_json::from_str(EXAMPLE_TRACE_ARR).unwrap()
    => ::serde_json::from_str::<Vec<Trace>>(EXAMPLE_TRACE_ARR).unwrap()
    );

    rpc_test!(
    Traces:get, "0000000000000000000000000000000000000000000000000000000000000123".parse::<H256>().unwrap(), vec![0.into()]
    =>
    "trace_get", vec![r#""0x0000000000000000000000000000000000000000000000000000000000000123""#, r#"["0x0"]"#];
    ::serde_json::from_str(EXAMPLE_TRACE).unwrap()
    => ::serde_json::from_str::<Trace>(EXAMPLE_TRACE).unwrap()
    );

    rpc_test!(
    Traces:transaction, "0000000000000000000000000000000000000000000000000000000000000123".parse::<H256>().unwrap()
    =>
    "trace_transaction", vec![r#""0x0000000000000000000000000000000000000000000000000000000000000123""#];
    ::serde_json::from_str(EXAMPLE_TRACE_ARR).unwrap()
    => ::serde_json::from_str::<Vec<Trace>>(EXAMPLE_TRACE_ARR).unwrap()
    );
}
