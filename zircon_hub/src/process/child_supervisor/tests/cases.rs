use std::io;
use std::process::ExitStatus;
use std::time::{Duration, Instant};

use super::{terminate_direct_child_and_reap_until, DirectChildControl};

struct StubbornChild {
    kill_count: usize,
    poll_count: usize,
}

impl DirectChildControl for StubbornChild {
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.poll_count += 1;
        Ok(None)
    }

    fn kill(&mut self) -> io::Result<()> {
        self.kill_count += 1;
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "kill denied",
        ))
    }
}

#[test]
fn denied_kill_with_stubborn_child_stops_polling_at_deadline() {
    let mut child = StubbornChild {
        kill_count: 0,
        poll_count: 0,
    };
    let started = Instant::now();
    let error =
        terminate_direct_child_and_reap_until(&mut child, started + Duration::from_millis(40))
            .expect_err("failed kill and live child must report incomplete reap");

    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(child.kill_count, 1);
    assert!(child.poll_count > 1);
    assert!(error.to_string().contains("kill denied"));
    assert!(error
        .to_string()
        .contains("direct child did not exit before its reap deadline"));
}
