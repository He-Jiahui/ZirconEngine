use super::*;
use crate::core::resource::ResourceId;

#[test]
fn dynamic_scene_asset_reload_order_keeps_one_physical_slot_per_asset() {
    let asset_id = ResourceId::from_stable_label("reload-order");
    let mut order = AssetIdOrder::default();

    for _ in 0..10_000 {
        order.push_back(asset_id);
    }

    assert_eq!(order.len(), 1);
    assert_eq!(order.pop_front(), Some(asset_id));
    assert!(order.is_empty());
}

#[test]
fn dynamic_scene_asset_reload_asset_order_scale_matrix_keeps_linear_physical_work() {
    for asset_count in [1usize, 1_000, 100_000] {
        let mut order = AssetIdOrder::default();
        let asset_ids = (0..asset_count)
            .map(|index| ResourceId::from_stable_label(&format!("reload-scale-{index}")))
            .collect::<Vec<_>>();

        for asset_id in &asset_ids {
            assert!(order.push_back(*asset_id));
            assert!(!order.push_back(*asset_id));
        }
        assert_eq!(order.len(), asset_count);

        for expected in asset_ids {
            assert_eq!(order.pop_front(), Some(expected));
        }
        assert!(order.is_empty());
    }
}
