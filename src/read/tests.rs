use super::*;
use alloy::{
    primitives::{Bytes, FixedBytes},
    providers::ProviderBuilder,
    rpc::client::RpcClient,
    sol_types::SolCall,
    transports::mock::{Asserter, MockTransport},
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower::Service;

type Requests = Arc<Mutex<Vec<Value>>>;

fn mocked_provider() -> (impl Provider + Clone, Asserter, Requests) {
    let responses = Asserter::new();
    let requests = Requests::default();
    let recorded = requests.clone();
    let mock = MockTransport::new(responses.clone());
    let transport = tower::service_fn(move |request| {
        recorded
            .lock()
            .unwrap()
            .push(serde_json::to_value(&request).unwrap());
        mock.clone().call(request)
    });
    let provider = ProviderBuilder::new().connect_client(RpcClient::new(transport, true));
    (provider, responses, requests)
}

#[tokio::test]
async fn storage_reads_preserve_the_block_and_typed_errors() {
    let (provider, responses, requests) = mocked_provider();
    let proxy = Address::repeat_byte(0x22);
    let implementation = Address::repeat_byte(0x11);
    let dispatch = Dispatch::Storage(U256::from(42));
    responses.push_success(&U256::from_be_bytes(implementation.into_word().0));
    let result = get_proxy_implementation(provider.clone(), &proxy, &dispatch, Some(123))
        .await
        .unwrap();
    assert!(matches!(result, ProxyImplementation::Single(address) if address == implementation));
    {
        let requests = requests.lock().unwrap();
        assert_eq!(requests[0]["method"], "eth_getStorageAt");
        assert_eq!(requests[0]["params"], json!([proxy, "0x2a", "0x7b"]));
    }

    responses.push_success(&U256::MAX);
    let result = get_proxy_implementation(provider.clone(), &proxy, &dispatch, None).await;
    assert!(matches!(result, Err(ProxyReadError::StorageNotAddress)));

    responses.push_failure_msg("offline failure");
    let result = get_proxy_implementation(provider, &proxy, &dispatch, Some(123)).await;
    assert!(
        matches!(result, Err(ProxyReadError::RPCError(message)) if message.contains("offline failure"))
    );
    assert!(responses.read_q().is_empty());
}

#[tokio::test]
async fn multiple_storage_and_self_address_dispatches_use_alloy2() {
    let (provider, responses, requests) = mocked_provider();
    let proxy = Address::repeat_byte(0x22);
    let implementations = [Address::repeat_byte(1), Address::repeat_byte(2)];
    for implementation in implementations {
        responses.push_success(&U256::from_be_bytes(implementation.into_word().0));
    }
    let result = get_proxy_implementation(
        provider.clone(),
        &proxy,
        &Dispatch::MultipleStorage(vec![U256::from(1), U256::from(2)]),
        Some(123),
    )
    .await
    .unwrap();
    assert!(
        matches!(result, ProxyImplementation::Multiple(addresses) if addresses == implementations)
    );

    responses.push_success(&U256::from_be_bytes(implementations[0].into_word().0));
    let result = get_proxy_implementation(provider, &proxy, &Dispatch::SelfAddressSlot, Some(123))
        .await
        .unwrap();
    assert!(
        matches!(result, ProxyImplementation::Single(address) if address == implementations[0])
    );
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests[2]["params"],
        json!([proxy, U256::from_be_bytes(proxy.into_word().0), "0x7b"])
    );
    assert!(responses.read_q().is_empty());
}

#[tokio::test]
async fn diamond_facets_decode_and_keep_the_requested_block() {
    let (provider, responses, requests) = mocked_provider();
    let proxy = Address::repeat_byte(0x22);
    let implementation = Address::repeat_byte(0x11);
    let facets = vec![IDiamondLoupe::Facet {
        facetAddress: implementation,
        functionSelectors: vec![FixedBytes::from([0x11, 0x22, 0x33, 0x44])],
    }];
    responses.push_success(&Bytes::from(IDiamondLoupe::facetsCall::abi_encode_returns(
        &facets,
    )));
    let result = get_proxy_implementation(provider, &proxy, &Dispatch::DiamondFacets, Some(123))
        .await
        .unwrap();
    assert!(
        matches!(result, ProxyImplementation::Facets(facets) if facets == HashMap::from([(implementation, 0x4433_2211)]))
    );
    let request = &requests.lock().unwrap()[0];
    assert_eq!(request["method"], "eth_call");
    assert_eq!(request["params"][0]["to"], json!(proxy));
    assert_eq!(request["params"][1], "0x7b");
    assert!(responses.read_q().is_empty());
}
