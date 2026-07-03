use std::fs;
use std::path::Path;

const MAX_TEST_MODULE_LINES: usize = 250;
const EXPECTED_TEST_MODULES: &[&str] = &[
    "accessibility",
    "api_table",
    "host_request_payloads",
    "host_requests",
    "input_events",
    "profile_control",
    "session_entry_points",
    "session_lifecycle",
    "session_profiles",
    "structure",
    "support",
    "viewport",
];

#[test]
fn dynamic_api_tests_stay_folder_backed_by_behavior_owner() {
    let dynamic_api_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("dynamic_api");
    let legacy_tests_file = dynamic_api_root.join("tests.rs");
    assert!(
        !legacy_tests_file.exists(),
        "dynamic API coverage must stay in src/dynamic_api/tests/, not {:?}",
        legacy_tests_file
    );

    let tests_dir = dynamic_api_root.join("tests");
    assert!(
        tests_dir.is_dir(),
        "dynamic API test tree is missing: {:?}",
        tests_dir
    );

    let mod_source =
        fs::read_to_string(tests_dir.join("mod.rs")).expect("dynamic_api/tests/mod.rs is readable");
    for module in EXPECTED_TEST_MODULES {
        let module_path = tests_dir.join(format!("{module}.rs"));
        assert!(
            module_path.exists(),
            "dynamic API test owner module is missing: {:?}",
            module_path
        );
        assert!(
            mod_source.contains(&format!("mod {module};")),
            "dynamic_api/tests/mod.rs must declare `mod {module};`"
        );

        let module_source = fs::read_to_string(&module_path)
            .unwrap_or_else(|error| panic!("failed to read {:?}: {error}", module_path));
        let line_count = module_source.lines().count();
        assert!(
            line_count <= MAX_TEST_MODULE_LINES,
            "{:?} has {line_count} lines; split this test owner before it becomes another monolith",
            module_path
        );
    }

    let non_declaration_lines: Vec<_> = mod_source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("mod "))
        .collect();
    assert!(
        non_declaration_lines.is_empty(),
        "dynamic_api/tests/mod.rs must stay navigational; found {:?}",
        non_declaration_lines
    );
}
