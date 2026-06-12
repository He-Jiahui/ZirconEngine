use crossbeam_channel::unbounded;

use crate::core::framework::channel::recv_latest;

#[test]
fn recv_latest_keeps_last_message() {
    let (sender, receiver) = unbounded();
    sender.send(1).unwrap();
    sender.send(2).unwrap();

    assert_eq!(recv_latest(&receiver), Some(2));
    assert_eq!(recv_latest::<i32>(&receiver), None);
}
