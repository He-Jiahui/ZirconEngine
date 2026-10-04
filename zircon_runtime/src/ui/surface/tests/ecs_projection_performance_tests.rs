#[test]
fn ecs_projection_reserves_the_retained_node_upper_bound() {
    let source = include_str!("../ecs_projection.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("ECS projection production source");

    assert!(
        production.contains("let mut nodes = Vec::with_capacity(self.tree.nodes.len());"),
        "ECS node projection should reserve the retained tree node count"
    );
    assert!(
        production.contains("nodes.push(UiEcsNodeProjection {"),
        "ECS node projection should preserve explicit one-pass construction"
    );
    assert!(
        production.contains("let flags = component_state.map(|state| &state.flags);"),
        "interaction projection should borrow component flags"
    );
    assert!(
        !production.contains("state.flags.clone()"),
        "interaction projection should not clone component flags per node"
    );
}
