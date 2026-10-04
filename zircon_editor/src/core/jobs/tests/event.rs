use std::sync::Arc;

use super::{JobEvent, JobEventKind};
use crate::core::jobs::{JobCategory, JobId};

#[test]
fn cloned_events_share_the_job_stable_label_allocation() {
    let event = JobEvent::new(
        JobId::new(7),
        Arc::<str>::from("thumbnail-stable-label"),
        JobCategory::Thumbnail,
        JobEventKind::Started,
    );

    let cloned = event.clone();

    assert_eq!(event.label(), "thumbnail-stable-label");
    assert_eq!(cloned.label(), event.label());
    assert!(Arc::ptr_eq(&event.label, &cloned.label));
}
