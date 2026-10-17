use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use super::{lock_children, EditorChildReaper};
use crate::process::SupervisedChild;

#[test]
fn one_reaper_owns_and_collects_ready_editor_children() {
    let reaper = EditorChildReaper::start().expect("start child reaper");
    let child = finished_child();
    reaper.register(1, child).expect("register child");

    let deadline = Instant::now() + Duration::from_secs(2);
    while !lock_children(&reaper.inner).is_empty() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }

    assert!(lock_children(&reaper.inner).is_empty());
}

#[cfg(windows)]
fn finished_child() -> SupervisedChild {
    let mut command = Command::new("cmd");
    command.args(["/C", "exit", "0"]);
    SupervisedChild::spawn(&mut command, "finished Editor fixture").expect("spawn fixture child")
}

#[cfg(unix)]
fn finished_child() -> SupervisedChild {
    let mut command = Command::new("sh");
    command.args(["-c", "exit 0"]);
    SupervisedChild::spawn(&mut command, "finished Editor fixture").expect("spawn fixture child")
}
