use super::{TaaResolveBindGroupKey, MAX_TAA_RESOLVE_BIND_GROUPS};
use crate::graphics::resource_identity::SampledTextureIdentity;

#[test]
fn taa_resolve_bind_group_cache_is_bounded() {
    assert_eq!(MAX_TAA_RESOLVE_BIND_GROUPS, 8);
}

#[test]
fn taa_resolve_key_rejects_each_sampled_view_change() {
    let base = [
        SampledTextureIdentity::new(),
        SampledTextureIdentity::new(),
        SampledTextureIdentity::new(),
        SampledTextureIdentity::new(),
        SampledTextureIdentity::new(),
    ];
    let key = TaaResolveBindGroupKey::new(base[0], base[1], base[2], base[3], base[4]);

    for changed_index in 0..base.len() {
        let mut changed = base;
        changed[changed_index] = SampledTextureIdentity::new();
        assert_ne!(
            key,
            TaaResolveBindGroupKey::new(changed[0], changed[1], changed[2], changed[3], changed[4],)
        );
    }
}

#[test]
fn history_pair_identity_is_order_independent_but_rejects_recreation() {
    let first = SampledTextureIdentity::new();
    let second = SampledTextureIdentity::new();

    assert_eq!(
        super::TaaResolveHistoryPairKey::new(first, second),
        super::TaaResolveHistoryPairKey::new(second, first)
    );
    assert_ne!(
        super::TaaResolveHistoryPairKey::new(first, second),
        super::TaaResolveHistoryPairKey::new(first, SampledTextureIdentity::new())
    );
}

#[test]
fn clear_forgets_target_and_history_generations() {
    let first = SampledTextureIdentity::new();
    let second = SampledTextureIdentity::new();
    let third = SampledTextureIdentity::new();
    let mut cache = super::TaaResolveBindGroupCache {
        entries: Default::default(),
        frame_target: Some(super::TaaResolveFrameTargetKey {
            scene_color: first,
            scene_depth: second,
            scene_velocity: third,
        }),
        history_pair: Some(super::TaaResolveHistoryPairKey::new(first, second)),
    };

    cache.clear();

    assert!(cache.entries.is_empty());
    assert_eq!(cache.frame_target, None);
    assert_eq!(cache.history_pair, None);
}
