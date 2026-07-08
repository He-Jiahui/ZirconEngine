pub(in super::super) fn assert_documents_and_maps(
    label: &str,
    anchors: &[&str],
    split_name: &str,
    split_id: &str,
    split_date: &str,
) {
    super::super::status_documents::assert_status_documents_contain(label, anchors);
    super::super::status_maps::assert_status_maps_contain(label, split_name, split_id, split_date);
}
