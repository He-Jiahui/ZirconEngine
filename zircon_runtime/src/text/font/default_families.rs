//! 字体数据库的候选家族顺序；共享运行时另有打包字体兜底，系统字体仅按策略显式发现。
use crate::text::FontFamilyName;

const DEFAULT_RUNTIME_FONT_FAMILIES: [&str; 5] = [
    "Inter",
    "Noto Sans",
    "Noto Sans CJK SC",
    "Microsoft YaHei UI",
    "Segoe UI",
];

pub(crate) fn default_runtime_font_families() -> Vec<FontFamilyName> {
    DEFAULT_RUNTIME_FONT_FAMILIES
        .iter()
        .map(|family| FontFamilyName::from(*family))
        .collect()
}
