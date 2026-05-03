use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let vendor = PathBuf::from("vendor/recastnavigation");
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .include(vendor.join("Recast/Include"))
        .include(vendor.join("Detour/Include"))
        .include(vendor.join("DetourCrowd/Include"))
        .include(vendor.join("DetourTileCache/Include"))
        .file("native/recast_bridge.cpp")
        .file("native/recast_bake.cpp");

    for directory in [
        vendor.join("Recast/Source"),
        vendor.join("Detour/Source"),
        vendor.join("DetourCrowd/Source"),
        vendor.join("DetourTileCache/Source"),
    ] {
        add_cpp_sources(&mut build, &directory);
    }

    if build.get_compiler().is_like_msvc() {
        build.flag_if_supported("/std:c++17");
    } else {
        build.flag_if_supported("-std=c++17");
    }

    build.compile("zircon_navigation_recast_bridge");

    println!("cargo:rerun-if-changed=native/recast_bridge.cpp");
    println!("cargo:rerun-if-changed=native/recast_bridge.h");
    println!("cargo:rerun-if-changed=native/recast_bake.cpp");
    println!("cargo:rerun-if-changed=vendor/recastnavigation");
}

fn add_cpp_sources(build: &mut cc::Build, directory: &Path) {
    for entry in fs::read_dir(directory).expect("vendored Recast/Detour source directory exists") {
        let path = entry
            .expect("vendored Recast/Detour source entry is readable")
            .path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("cpp") {
            build.file(path);
        }
    }
}
