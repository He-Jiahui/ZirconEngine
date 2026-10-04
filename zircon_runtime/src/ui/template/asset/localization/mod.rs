//! 为编译前合法性检查、包依赖和 Editor 翻译面板提供源级本地化报告。
//! 目录校验验证 locale/table/key 是否已登记；不在这里替换运行时显示文本。

mod collect;
mod resolve;

pub use collect::{collect_document_localization_report, validate_document_localization};
pub use resolve::{
    localization_table_keys_from_toml_str, validate_localization_report_against_catalog,
    UiLocalizationTableCatalog,
};
