// 对照前排序是为消除声明和组装顺序差异；依赖、目录与模块身份仍逐项比较。
mod collect;
mod ordering;

use super::StaticSoundContributions;

pub(in crate::tests::manifest::support) fn static_sound_contributions(
    manifest: &str,
) -> StaticSoundContributions {
    let mut contributions = collect::static_sound_contributions_from_plugin_toml(manifest);
    ordering::sort_static_sound_contributions(&mut contributions);
    contributions
}
