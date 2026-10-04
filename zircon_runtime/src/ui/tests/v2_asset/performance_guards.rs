//! 记录运行态样式和树构建的性能回归守卫；该守卫读取生产源码文本。
// TODO: [CR-UI-TEST-0002] 确认源码字符串守卫能否覆盖等价的逐子节点复制形式；当前仅排除固定拼写，下一步改用语法或行为证据。
#[test]
fn runtime_style_traversal_does_not_clone_each_nodes_children() {
    let source = include_str!("../../v2/style.rs");

    assert!(
        !source.contains(".children\n                .clone()"),
        "runtime pseudo-state traversal must borrow the next child instead of cloning every child list"
    );
}

#[test]
fn surface_tree_build_moves_owned_slot_attributes_into_metadata() {
    let source = include_str!("../../v2/surface_tree/node.rs");

    assert!(
        !source.contains("frame.slot.clone()"),
        "the build frame already owns slot attributes and must move them into the tree"
    );
    assert!(
        !source.contains("slot_attributes: slot_attributes.clone()"),
        "tree metadata must take ownership of slot attributes after inference"
    );
}
