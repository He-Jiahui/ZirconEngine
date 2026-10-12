use super::*;
use crate::ui::workbench::snapshot::AssetWorkspaceItemGeneration;

type ClickContext = (u64, AssetViewMode);

fn register(
    tracker: &mut AssetActivationClickTracker,
    asset: Option<(&str, &str, Option<u64>)>,
    items: &AssetWorkspaceItemGeneration,
    context: ClickContext,
    at: Instant,
) -> bool {
    let (asset_uuid, asset_locator, revision) = asset
        .map(|(uuid, locator, revision)| (Some(uuid), Some(locator), revision))
        .unwrap_or((None, None, None));
    tracker.register_click(
        asset_uuid,
        asset_locator,
        revision,
        items,
        context.0,
        context.1,
        at,
    )
}

#[test]
fn activation_requires_same_asset_identity_and_list_context_within_window() {
    let start = Instant::now();
    let items = AssetWorkspaceItemGeneration::default();
    let rebuilt_items = AssetWorkspaceItemGeneration::default();
    let mut tracker = AssetActivationClickTracker::default();
    let context = (7, AssetViewMode::List);

    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a.zag", Some(1))),
        &items,
        context,
        start
    ));
    assert!(register(
        &mut tracker,
        Some(("asset-a", "res://asset-a.zag", Some(1))),
        &items,
        context,
        start + Duration::from_millis(500)
    ));

    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a.zag", Some(1))),
        &items,
        context,
        start + Duration::from_millis(600)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-b", "res://asset-b.zag", Some(1))),
        &items,
        context,
        start + Duration::from_millis(700)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a.zag", Some(1))),
        &items,
        context,
        start + Duration::from_millis(800)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(1))),
        &items,
        context,
        start + Duration::from_millis(900)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(2))),
        &items,
        context,
        start + Duration::from_millis(1_000)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(2))),
        &items,
        (8, AssetViewMode::List),
        start + Duration::from_millis(1_100)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(2))),
        &items,
        (8, AssetViewMode::List),
        start + Duration::from_millis(1_200)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(2))),
        &items,
        (8, AssetViewMode::List),
        start + Duration::from_millis(1_300)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(2))),
        &items,
        (8, AssetViewMode::Thumbnail),
        start + Duration::from_millis(1_400)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(2))),
        &rebuilt_items,
        (8, AssetViewMode::Thumbnail),
        start + Duration::from_millis(1_500)
    ));
    assert!(!register(
        &mut tracker,
        Some(("asset-a", "res://asset-a-v2.zag", Some(2))),
        &items,
        context,
        start + Duration::from_millis(2_001)
    ));
}

#[test]
fn blank_target_clears_candidate_and_surfaces_keep_independent_trackers() {
    let start = Instant::now();
    let items = AssetWorkspaceItemGeneration::default();
    let context = (7, AssetViewMode::List);
    let asset = Some(("asset-a", "res://asset-a.zag", Some(1)));
    let mut browser = AssetActivationClickTracker::default();
    let mut activity = AssetActivationClickTracker::default();

    assert!(!register(&mut browser, asset, &items, context, start));
    assert!(!register(
        &mut activity,
        asset,
        &items,
        context,
        start + Duration::from_millis(100)
    ));
    assert!(!register(
        &mut browser,
        None,
        &items,
        context,
        start + Duration::from_millis(150)
    ));
    assert!(!register(
        &mut browser,
        asset,
        &items,
        context,
        start + Duration::from_millis(200)
    ));
    assert!(register(
        &mut activity,
        asset,
        &items,
        context,
        start + Duration::from_millis(250)
    ));
}
