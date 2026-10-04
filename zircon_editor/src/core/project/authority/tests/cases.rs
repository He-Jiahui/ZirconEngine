//! 守住已选定物理项目身份后的打开路径，避免低层重复链接检查把别名解析后的合法身份误判；这是源码形式守卫。
#[test]
fn canonical_project_resolution_does_not_repeat_link_component_validation() {
    let source = include_str!("../open_project.rs");
    let repeated_validation = ["validate_existing_project_root", "(&root)"].concat();

    assert!(!source.contains(&repeated_validation));
}
