use crate::scene::components::Name;
use crate::scene::ecs::{Component, QuerySingleError, QueryState, SystemState};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[test]
fn query_state_single_mut_reports_zero_one_many_and_mutates_match() {
    let mut world = World::empty();
    let mut query = world.query::<&mut Health>();

    assert_eq!(
        query.single_mut(&mut world).map(|health| health.0),
        Err(QuerySingleError::NoEntities)
    );

    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    {
        let health = query.single_mut(&mut world).unwrap();
        health.0 += 1;
    }
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));

    world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    assert_eq!(
        query.single_mut(&mut world).map(|health| health.0),
        Err(QuerySingleError::MultipleEntities)
    );
}

#[test]
fn system_query_single_mut_reports_zero_one_many_and_uses_run_window() {
    let mut world = World::empty();
    type HealthQuery = QueryState<&'static mut Health>;
    let mut system = SystemState::<HealthQuery>::new(&mut world).unwrap();

    let empty = system.run(&mut world, |mut query| {
        query.single_mut().map(|health| health.0)
    });
    assert_eq!(empty, Err(QuerySingleError::NoEntities));

    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let one = system.run(&mut world, |mut query| {
        let health = query.single_mut().unwrap();
        health.0 += 5;
        health.0
    });
    assert_eq!(one, 15);
    assert_eq!(world.get::<Health>(first), Some(&Health(15)));

    world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let many = system.run(&mut world, |mut query| {
        query.single_mut().map(|health| health.0)
    });
    assert_eq!(many, Err(QuerySingleError::MultipleEntities));
}
