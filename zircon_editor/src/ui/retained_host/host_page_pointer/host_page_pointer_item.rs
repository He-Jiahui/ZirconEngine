#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostPagePointerItem {
    pub page_id: String,
    pub title: String,
    pub close_instance_id: Option<String>,
}
