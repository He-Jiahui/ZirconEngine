use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementPageRequest {
    #[serde(default)]
    pub offset: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementPageInfo {
    #[serde(default)]
    pub offset: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    pub total_count: usize,
    pub returned_count: usize,
    pub has_previous_page: bool,
    pub has_next_page: bool,
}

impl RenderMaterialManagementPageRequest {
    pub fn new(offset: usize, limit: Option<usize>) -> Self {
        Self { offset, limit }
    }

    pub fn all() -> Self {
        Self::default()
    }
}

impl RenderMaterialManagementPageInfo {
    pub fn from_page_request(
        request: RenderMaterialManagementPageRequest,
        total_count: usize,
        returned_count: usize,
    ) -> Self {
        let has_previous_page = request.offset > 0 && total_count > 0;
        let has_next_page = request.offset.saturating_add(returned_count) < total_count;
        Self {
            offset: request.offset,
            limit: request.limit,
            total_count,
            returned_count,
            has_previous_page,
            has_next_page,
        }
    }
}
