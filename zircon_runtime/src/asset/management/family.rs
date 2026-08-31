use super::{
    AssetManagementFamilyIssueBucket, AssetManagementFamilyIssueIndex,
    AssetManagementFamilyIssueView, AssetManagementFamilyKind, AssetManagementFamilyStatus,
    AssetManagementFamilyStatusIndex, AssetManagementFamilyStatusView,
    AssetManagementFamilySummary,
};

impl AssetManagementFamilySummary {
    pub fn new(
        kind: AssetManagementFamilyKind,
        total_record_count: usize,
        ready_record_count: usize,
        degraded_record_count: usize,
        issue_row_count: usize,
    ) -> Self {
        let status = if total_record_count == 0 {
            AssetManagementFamilyStatus::Empty
        } else if degraded_record_count > 0 {
            AssetManagementFamilyStatus::Degraded
        } else {
            AssetManagementFamilyStatus::Ready
        };
        Self {
            kind,
            status,
            total_record_count,
            ready_record_count,
            degraded_record_count,
            issue_row_count,
        }
    }
}

impl AssetManagementFamilyStatus {
    fn matches(self, family: &AssetManagementFamilySummary) -> bool {
        family.status == self
    }
}

impl AssetManagementFamilyStatusIndex {
    pub fn from_families(families: &[AssetManagementFamilySummary]) -> Self {
        let mut index = Self::default();
        for family in families {
            match family.status {
                AssetManagementFamilyStatus::Empty => index.empty.push(family.kind),
                AssetManagementFamilyStatus::Ready => index.ready.push(family.kind),
                AssetManagementFamilyStatus::Degraded => index.degraded.push(family.kind),
            }
        }
        index
    }

    pub fn total_family_count(&self) -> usize {
        self.empty.len() + self.ready.len() + self.degraded.len()
    }

    pub fn degraded_family_count(&self) -> usize {
        self.degraded.len()
    }

    pub fn has_degraded_families(&self) -> bool {
        !self.degraded.is_empty()
    }

    pub fn families_for_status(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> &[AssetManagementFamilyKind] {
        match status {
            AssetManagementFamilyStatus::Empty => &self.empty,
            AssetManagementFamilyStatus::Ready => &self.ready,
            AssetManagementFamilyStatus::Degraded => &self.degraded,
        }
    }
}

impl AssetManagementFamilyStatusView {
    pub fn from_families(
        families: &[AssetManagementFamilySummary],
        status: AssetManagementFamilyStatus,
    ) -> Self {
        let rows = families
            .iter()
            .filter(|family| status.matches(family))
            .cloned()
            .collect::<Vec<_>>();
        let total_record_count = rows.iter().map(|family| family.total_record_count).sum();
        let ready_record_count = rows.iter().map(|family| family.ready_record_count).sum();
        let degraded_record_count = rows.iter().map(|family| family.degraded_record_count).sum();
        let issue_row_count = rows.iter().map(|family| family.issue_row_count).sum();
        let families = rows.iter().map(|family| family.kind).collect();
        Self {
            status,
            families,
            rows,
            total_record_count,
            ready_record_count,
            degraded_record_count,
            issue_row_count,
        }
    }
}

impl AssetManagementFamilyIssueBucket {
    fn matches(self, family: &AssetManagementFamilySummary) -> bool {
        match self {
            Self::Clean => family.issue_row_count == 0,
            Self::WithIssues => family.issue_row_count > 0,
        }
    }
}

impl AssetManagementFamilyIssueIndex {
    pub fn from_families(families: &[AssetManagementFamilySummary]) -> Self {
        let mut index = Self::default();
        for family in families {
            if family.issue_row_count > 0 {
                index.with_issues.push(family.kind);
            } else {
                index.clean.push(family.kind);
            }
        }
        index
    }

    pub fn total_family_count(&self) -> usize {
        self.clean.len() + self.with_issues.len()
    }

    pub fn issue_family_count(&self) -> usize {
        self.with_issues.len()
    }

    pub fn has_issue_families(&self) -> bool {
        !self.with_issues.is_empty()
    }

    pub fn families_with_issues(&self) -> &[AssetManagementFamilyKind] {
        &self.with_issues
    }

    pub fn families_without_issues(&self) -> &[AssetManagementFamilyKind] {
        &self.clean
    }

    pub fn families_for_bucket(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> &[AssetManagementFamilyKind] {
        match bucket {
            AssetManagementFamilyIssueBucket::Clean => &self.clean,
            AssetManagementFamilyIssueBucket::WithIssues => &self.with_issues,
        }
    }
}

impl AssetManagementFamilyIssueView {
    pub fn from_families(
        families: &[AssetManagementFamilySummary],
        bucket: AssetManagementFamilyIssueBucket,
    ) -> Self {
        let rows = families
            .iter()
            .filter(|family| bucket.matches(family))
            .cloned()
            .collect::<Vec<_>>();
        let issue_row_count = rows.iter().map(|family| family.issue_row_count).sum();
        let families = rows.iter().map(|family| family.kind).collect();
        Self {
            bucket,
            families,
            rows,
            issue_row_count,
        }
    }
}
