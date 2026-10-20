pub(super) fn assert_current_inventory(mirror_anchors: &[&str]) {
    // Active Runtime13 planning and runtime-index tracking are not current contract mirrors.
    // The domain ledger and these two architectural consumers retain independent inventories.
    let mirror_docs = [
        (
            "Runtime 13 function ledger",
            include_str!(
                "../../../../../../docs/crates/zircon_runtime/script/vm/host/function_ledger.md"
            ),
        ),
        (
            "M0 review",
            include_str!("../../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../../../docs/architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        let inventories = doc_source
            .lines()
            .filter(|line| line.starts_with("Runtime13 current source contract inventory:"))
            .collect::<Vec<_>>();
        assert_eq!(
            inventories.len(),
            1,
            "{doc_name} needs one current inventory"
        );
        assert_contains_all(
            doc_name,
            inventories[0],
            &[
                "fixed_host_function_count = 63",
                "builtin_callback_count = 21",
                "gameplay_callback_count = 40",
                "host_capability_count = 13",
            ],
        );
        let unlabelled = unlabelled_host_inventories(doc_source);
        assert!(
            unlabelled.is_empty(),
            "{doc_name} must label old inventories as historical: {unlabelled:?}"
        );
        assert_contains_all(doc_name, doc_source, mirror_anchors);
    }
}

pub(super) fn unlabelled_host_inventories(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| {
            !line.starts_with("Runtime13 current source contract inventory:")
                && !line.contains("Historical Runtime13 source snapshot")
                && [
                    "fixed_host_function_count",
                    "builtin_callback_count",
                    "gameplay_callback_count",
                    "host_capability_count",
                ]
                .iter()
                .any(|metric| line.contains(metric))
        })
        .collect()
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should mirror Runtime 13 script-binding audit anchor `{anchor}`"
        );
    }
}
