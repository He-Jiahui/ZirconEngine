// 静态依赖对照键保留 ID、必需性和能力约束，与包清单依赖同域比较。
pub(in crate::tests::manifest::support::contributions) type StaticDependency =
    (String, bool, Option<String>);
