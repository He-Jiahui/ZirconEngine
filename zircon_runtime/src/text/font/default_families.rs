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
