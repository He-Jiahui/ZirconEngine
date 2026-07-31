use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let report = woc_runtime::identity_report_json(root, woc_runtime::WocHostRole::Server)
        .expect("WOC server project identity must be valid");
    println!("{report}");
}
