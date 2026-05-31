use super::{
    RenderMaterialManagementPageInfo, RenderMaterialManagementPageRequest,
    RenderMaterialManagementQuery,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementPageWindow {
    #[serde(default)]
    pub start_index: usize,
    #[serde(default)]
    pub end_index_exclusive: usize,
}

impl RenderMaterialManagementPageWindow {
    pub fn len(&self) -> usize {
        self.end_index_exclusive.saturating_sub(self.start_index)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn display_start_index(&self) -> Option<usize> {
        (!self.is_empty()).then_some(self.start_index.saturating_add(1))
    }

    pub fn display_end_index(&self) -> Option<usize> {
        (!self.is_empty()).then_some(self.end_index_exclusive)
    }
}

impl RenderMaterialManagementPageInfo {
    pub fn page_request(&self) -> RenderMaterialManagementPageRequest {
        RenderMaterialManagementPageRequest::new(self.offset, self.limit)
    }

    pub fn window(&self) -> RenderMaterialManagementPageWindow {
        let start_index = self.offset.min(self.total_count);
        let end_index_exclusive = start_index
            .saturating_add(self.returned_count)
            .min(self.total_count);
        RenderMaterialManagementPageWindow {
            start_index,
            end_index_exclusive,
        }
    }

    pub fn display_start_index(&self) -> Option<usize> {
        self.window().display_start_index()
    }

    pub fn display_end_index(&self) -> Option<usize> {
        self.window().display_end_index()
    }

    pub fn current_page_number(&self) -> Option<usize> {
        let limit = positive_page_limit(self.limit)?;
        (self.offset < self.total_count).then_some(self.offset / limit + 1)
    }

    pub fn total_page_count(&self) -> Option<usize> {
        let limit = positive_page_limit(self.limit)?;
        Some(if self.total_count == 0 {
            0
        } else {
            (self.total_count - 1) / limit + 1
        })
    }

    pub fn next_page_request(&self) -> Option<RenderMaterialManagementPageRequest> {
        let limit = positive_page_limit(self.limit)?;
        self.has_next_page.then(|| {
            RenderMaterialManagementPageRequest::new(self.offset.saturating_add(limit), self.limit)
        })
    }

    pub fn previous_page_request(&self) -> Option<RenderMaterialManagementPageRequest> {
        let limit = positive_page_limit(self.limit)?;
        self.has_previous_page.then(|| {
            RenderMaterialManagementPageRequest::new(self.offset.saturating_sub(limit), self.limit)
        })
    }
}

impl RenderMaterialManagementQuery {
    pub fn next_page_query(&self, page: RenderMaterialManagementPageInfo) -> Option<Self> {
        page.next_page_request()
            .map(|page_request| self.clone().with_page(page_request))
    }

    pub fn previous_page_query(&self, page: RenderMaterialManagementPageInfo) -> Option<Self> {
        page.previous_page_request()
            .map(|page_request| self.clone().with_page(page_request))
    }
}

fn positive_page_limit(limit: Option<usize>) -> Option<usize> {
    limit.filter(|limit| *limit > 0)
}
