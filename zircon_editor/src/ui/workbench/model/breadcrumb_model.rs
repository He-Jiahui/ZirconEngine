#[derive(Clone, Debug, PartialEq, Eq)]
/// 页头路径的显示片段；label不是文档或视图的可操作身份。
pub struct BreadcrumbModel {
    pub label: String,
}
