mod declarations;
mod extract_registration;
mod normalized_page_table_entries;
mod pending_completion;
mod plan_ingestion;
mod prepare_frame;
mod residency_management;
mod snapshot;
#[cfg(test)]
mod test_accessors;

pub(crate) use declarations::VirtualGeometryPageRequest;
#[cfg(test)]
pub(crate) use declarations::VirtualGeometryPageResidencyState;
pub(crate) use declarations::VirtualGeometryRuntimeState;
pub(crate) use normalized_page_table_entries::normalized_page_table_entries;
