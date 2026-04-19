use crossbeam_channel::{after, select, Receiver};
use notify::Event;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::{
    asset_change::AssetChange, asset_watcher::AssetWatcher, map_notify_event::map_notify_event,
};

const WATCH_DEBOUNCE: Duration = Duration::from_millis(120);

pub(super) fn watch_loop(
    assets_root: PathBuf,
    stop_rx: Receiver<()>,
    event_rx: Receiver<notify::Result<Event>>,
    on_changes: Arc<dyn Fn(Vec<AssetChange>) + Send + Sync>,
) {
    loop {
        select! {
            recv(stop_rx) -> _ => break,
            recv(event_rx) -> message => match message {
                Ok(Ok(event)) => {
                    let mut pending = map_notify_event(&assets_root, event);
                    if pending.is_empty() {
                        continue;
                    }
                    loop {
                        select! {
                            recv(stop_rx) -> _ => return,
                            recv(event_rx) -> next => match next {
                                Ok(Ok(event)) => pending.extend(map_notify_event(&assets_root, event)),
                                Ok(Err(_)) => {}
                                Err(_) => return,
                            },
                            recv(after(WATCH_DEBOUNCE)) -> _ => break,
                        }
                    }
                    let folded = AssetWatcher::fold_events(&pending);
                    if !folded.is_empty() {
                        on_changes(folded);
                    }
                }
                Ok(Err(_)) => {}
                Err(_) => break,
            }
        }
    }
}
