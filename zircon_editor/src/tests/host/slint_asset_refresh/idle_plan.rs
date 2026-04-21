use crate::ui::slint_host::{plan_asset_backend_refresh, AssetBackendRefreshPlan};

#[test]
fn asset_backend_refresh_plan_is_idle_without_backend_events() {
    let plan = plan_asset_backend_refresh(None, None, &[], &[], &[]);

    assert_eq!(plan, AssetBackendRefreshPlan::default());
}
