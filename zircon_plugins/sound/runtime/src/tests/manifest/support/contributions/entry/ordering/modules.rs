// 对已投影并过滤的 runtime 模块按身份排序，消除声明顺序对包清单等值对照的影响；此层不读取 TOML。
use super::super::super::StaticModule;

pub(super) fn sort_static_modules(modules: &mut [StaticModule]) {
    modules.sort_unstable_by(|left, right| left.0.cmp(&right.0));
}
