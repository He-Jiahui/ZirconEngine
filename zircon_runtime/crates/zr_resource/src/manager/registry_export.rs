use std::cmp::Ordering;
use std::sync::Arc;

use crate::{
    ResourceKind, ResourceLocator, ResourceManagementGeneration, ResourceManagementQuery,
    ResourceRecord, ResourceRegistry, ResourceScheme, ResourceState,
};

use super::resource_manager::ResourceManager;

// Private query-planner policy calibrated by the Windows-native registry-export preflight.
const MANAGEMENT_SCAN_LARGE_REGISTRY_MIN_RECORDS: usize = 32_768;
const MANAGEMENT_SCAN_MID_REGISTRY_MIN_RECORDS: usize = 4_096;
const MANAGEMENT_SCAN_VERY_SPARSE_READY_MAX_RECORDS: usize = 64;
const MANAGEMENT_SCAN_LARGE_REGISTRY_DENSITY_DIVISOR: usize = 10;
const MANAGEMENT_SCAN_MID_REGISTRY_DENSITY_DIVISOR: usize = 4;

// 注册表与管理投影必须取自同一次读取；后续排序和扫描脱离权威锁，不阻塞资源发布。
// 投影扫描不完整或与记录不一致时退回目录扫描，两条路径仍返回同一快照的结果。
struct ResourceRegistryExportSnapshot {
    registry: ResourceRegistry,
    management: Arc<ResourceManagementGeneration>,
}

impl ResourceRegistryExportSnapshot {
    fn capture(manager: &ResourceManager) -> Self {
        let authority = manager.lock_authority_read();
        Self {
            registry: authority.registry.clone(),
            management: authority.management.generation(),
        }
    }

    fn ready_records_for_kind(&self, kind: ResourceKind) -> Vec<ResourceRecord> {
        let summary = self.management.summary();
        let total_count = summary.total_count();
        let ready_count = summary.kind(kind).ready_count;
        if should_scan_management_generation(total_count, ready_count) {
            if let Some(records) = self.ready_records_from_management(kind, ready_count) {
                return records;
            }
        }
        self.ready_records_from_registry(kind, ready_count)
    }

    fn ready_records_from_registry(
        &self,
        kind: ResourceKind,
        ready_count: usize,
    ) -> Vec<ResourceRecord> {
        let mut records = Vec::with_capacity(ready_count);
        records.extend(
            self.registry
                .values()
                .filter(|record| ready_record_matches_kind(record, kind))
                .cloned(),
        );
        if records.len() > 1 {
            records.sort_unstable_by(|left, right| {
                compare_locators_by_canonical_display(&left.primary_locator, &right.primary_locator)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        records
    }

    fn ready_records_from_management(
        &self,
        kind: ResourceKind,
        ready_count: usize,
    ) -> Option<Vec<ResourceRecord>> {
        let mut scan = self.management.scan(ResourceManagementQuery {
            kind: Some(kind),
            state: Some(ResourceState::Ready),
        });
        let mut records = Vec::with_capacity(ready_count);
        let mut scanned_count = 0_usize;
        while let Some(row) = scan.next_row() {
            scanned_count = scanned_count.saturating_add(1);
            if row.revision == 0 {
                continue;
            }
            let record = self.registry.get(row.id)?;
            if !management_row_matches_record(&row, record) {
                return None;
            }
            records.push(record.clone());
        }
        (scan.is_complete() && scanned_count == ready_count).then_some(records)
    }
}

fn ready_record_matches_kind(record: &ResourceRecord, kind: ResourceKind) -> bool {
    record.kind == kind && record.state == ResourceState::Ready && record.revision != 0
}

fn management_row_matches_record(
    row: &crate::ResourceManagementRow,
    record: &ResourceRecord,
) -> bool {
    record.kind == row.kind
        && record.primary_locator.matches_display(&row.primary_locator)
        && record.revision == row.revision
        && record.state == row.state
}

fn compare_locators_by_canonical_display(
    left: &ResourceLocator,
    right: &ResourceLocator,
) -> Ordering {
    resource_scheme_display_name(left.scheme())
        .cmp(resource_scheme_display_name(right.scheme()))
        .then_with(|| left.path().cmp(right.path()))
        .then_with(|| left.label().cmp(&right.label()))
}

fn resource_scheme_display_name(scheme: ResourceScheme) -> &'static str {
    match scheme {
        ResourceScheme::Res => "res",
        ResourceScheme::Library => "lib",
        ResourceScheme::Package => "package",
        ResourceScheme::Builtin => "builtin",
        ResourceScheme::Memory => "mem",
    }
}

fn should_scan_management_generation(total_count: usize, ready_count: usize) -> bool {
    let large_registry_prefers_management = total_count
        >= MANAGEMENT_SCAN_LARGE_REGISTRY_MIN_RECORDS
        && (ready_count <= MANAGEMENT_SCAN_VERY_SPARSE_READY_MAX_RECORDS
            || ready_count.saturating_mul(MANAGEMENT_SCAN_LARGE_REGISTRY_DENSITY_DIVISOR)
                >= total_count);
    let mid_registry_prefers_management = total_count >= MANAGEMENT_SCAN_MID_REGISTRY_MIN_RECORDS
        && ready_count.saturating_mul(MANAGEMENT_SCAN_MID_REGISTRY_DENSITY_DIVISOR) >= total_count;
    large_registry_prefers_management || mid_registry_prefers_management
}

impl ResourceManager {
    /// 导出指定种类、已声明 Ready 且有有效版本的记录，按定位符显示顺序稳定排列，供预热和清单构建。
    /// 此处 Ready 表示元数据可加载；不要求运行时载荷驻留，也不会触发加载。
    pub fn ready_records_for_kind(&self, kind: ResourceKind) -> Vec<ResourceRecord> {
        ResourceRegistryExportSnapshot::capture(self).ready_records_for_kind(kind)
    }
}

#[cfg(test)]
#[path = "registry_export/tests/optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "tests/registry_export.rs"]
mod tests;
