use std::process::Command;

#[path = "asset_import_context/emission.rs"]
mod emission;
#[path = "asset_import_context/fingerprint.rs"]
mod fingerprint;

pub(super) fn emit() {
    println!("cargo:rerun-if-changed=build/asset_import_context.rs");
    println!("cargo:rerun-if-changed=build/asset_import_context/fingerprint.rs");
    println!("cargo:rerun-if-changed=build/asset_import_context/emission.rs");

    let configuration =
        fingerprint::canonical_configuration(std::env::vars_os().filter_map(|(name, value)| {
            let name = name.into_string().ok()?;
            if !fingerprint::is_compatibility_variable(&name) {
                return None;
            }
            let value = value
                .into_string()
                .unwrap_or_else(|_| panic!("Cargo compatibility variable {name} must be UTF-8"));
            Some((name, value))
        }));
    let rustc = std::env::var_os("RUSTC").expect("Cargo must provide the selected RUSTC");
    let compiler = Command::new(rustc)
        .arg("-vV")
        .output()
        .expect("query the compiler used to build asset importers");
    assert!(compiler.status.success(), "compiler identity query failed");
    for directive in emission::directives(&compiler.stdout, &configuration)
        .expect("valid Cargo inputs for asset import compatibility")
    {
        println!("{directive}");
    }
}
