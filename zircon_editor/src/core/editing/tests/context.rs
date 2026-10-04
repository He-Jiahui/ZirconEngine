use super::*;
use crate::core::play::PlayInstanceId;

#[test]
fn retiring_repeated_play_routes_keeps_selection_storage_bounded() {
    let mut context = CoreEditContext::default();

    for raw in 1..=256 {
        let world_domain = WorldDomain::Play(PlayInstanceId::for_test(raw));
        context
            .selections
            .insert(world_domain, SelectionSnapshot::fixture_value(raw, raw));

        context
            .retire_world_route(world_domain)
            .expect("an inactive play route should retire");
        assert_eq!(context.selections.len(), 1);
        assert!(context.selections.contains_key(&WorldDomain::Edit));
    }
}

#[test]
fn retiring_the_active_world_route_is_rejected() {
    let mut context = CoreEditContext::default();
    let world_domain = WorldDomain::Play(PlayInstanceId::for_test(1));
    context
        .selections
        .insert(world_domain, SelectionSnapshot::fixture_value(1, 1));
    context.active_route = Some(EditWorldRoute::logical(world_domain));

    assert!(matches!(
        context.retire_world_route(world_domain),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    assert!(context.selections.contains_key(&world_domain));
}
