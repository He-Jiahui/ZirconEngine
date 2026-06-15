use crate::{net_rpc_runtime_manager, RPC_CHANNEL_RELIABLE_ORDERED, RPC_CHANNEL_UNRELIABLE};

#[test]
fn channels_isolate_message_order() {
    let rpc = net_rpc_runtime_manager();

    let alpha_0 = rpc
        .enqueue_channel_message(1, RPC_CHANNEL_RELIABLE_ORDERED, b"alpha-0".to_vec())
        .unwrap();
    let beta_0 = rpc
        .enqueue_channel_message(2, RPC_CHANNEL_UNRELIABLE, b"beta-0".to_vec())
        .unwrap();
    let alpha_1 = rpc
        .enqueue_channel_message(1, RPC_CHANNEL_RELIABLE_ORDERED, b"alpha-1".to_vec())
        .unwrap();
    let beta_1 = rpc
        .enqueue_channel_message(2, RPC_CHANNEL_UNRELIABLE, b"beta-1".to_vec())
        .unwrap();

    assert_eq!(alpha_0.sequence, 0);
    assert_eq!(alpha_1.sequence, 1);
    assert_eq!(beta_0.sequence, 0);
    assert_eq!(beta_1.sequence, 1);

    let beta = rpc.drain_channel_messages(2, 8);
    assert_eq!(
        beta.iter()
            .map(|message| message.payload.as_slice())
            .collect::<Vec<_>>(),
        vec![b"beta-0".as_slice(), b"beta-1".as_slice()]
    );

    let alpha = rpc.drain_channel_messages(1, 8);
    assert_eq!(
        alpha
            .iter()
            .map(|message| message.payload.as_slice())
            .collect::<Vec<_>>(),
        vec![b"alpha-0".as_slice(), b"alpha-1".as_slice()]
    );
    assert!(alpha.iter().all(|message| message.is_reliable_ordered()));
    assert!(rpc.drain_channel_messages(1, 8).is_empty());
}
