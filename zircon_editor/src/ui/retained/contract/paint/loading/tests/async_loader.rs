use super::VisualAssetLoadScheduler;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn pending_loads_deduplicate_same_key() {
    let mut scheduler = VisualAssetLoadScheduler::default();

    assert!(scheduler.reserve_pending_key("icon:save@24x24", 7, None));
    assert!(!scheduler.reserve_pending_key("icon:save@24x24", 7, None));
    let _ = scheduler.release_pending_key("icon:save@24x24", 7);
    assert!(scheduler.reserve_pending_key("icon:save@24x24", 7, None));
}

#[test]
fn duplicate_pending_loads_union_damage_frames() {
    let mut scheduler = VisualAssetLoadScheduler::default();
    let first = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 30.0,
        height: 40.0,
    };
    let second = FrameRect {
        x: 50.0,
        y: 10.0,
        width: 20.0,
        height: 25.0,
    };

    assert!(scheduler.reserve_pending_key("icon:save@24x24", 7, Some(first)));
    assert!(!scheduler.reserve_pending_key("icon:save@24x24", 7, Some(second)));

    let damage = scheduler
        .pending_keys
        .get("icon:save@24x24")
        .and_then(|pending| pending.damage_frame.as_ref())
        .expect("duplicate visible users should retain a union damage frame");
    assert_eq!(
        damage,
        &FrameRect {
            x: 10.0,
            y: 10.0,
            width: 60.0,
            height: 50.0,
        }
    );
}

#[test]
fn unknown_damage_keeps_the_full_frame_completion_fallback() {
    let mut damage = Some(FrameRect {
        x: 10.0,
        y: 20.0,
        width: 30.0,
        height: 40.0,
    });

    super::merge_pending_damage_frame(&mut damage, None);

    assert!(damage.is_none());
}

#[test]
fn stale_binding_cannot_publish_visual_pixels() {
    let mut scheduler = VisualAssetLoadScheduler::default();
    scheduler.binding_epoch = 9;

    assert!(!scheduler.binding_is_current(8));
    assert!(scheduler.binding_is_current(9));
}
