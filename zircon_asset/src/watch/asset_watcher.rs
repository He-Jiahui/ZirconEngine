use crossbeam_channel::Sender;
use std::thread::JoinHandle;

#[derive(Debug)]
pub struct AssetWatcher {
    pub(super) stop_tx: Sender<()>,
    pub(super) join: Option<JoinHandle<()>>,
}
