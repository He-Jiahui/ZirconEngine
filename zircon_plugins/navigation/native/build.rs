use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let vendor = PathBuf::from("vendor/recastnavigation");
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .define("DT_VIRTUAL_QUERYFILTER", None)
        .include(vendor.join("Recast/Include"))
        .include(vendor.join("Detour/Include"))
        .include(vendor.join("DetourCrowd/Include"))
        .include(vendor.join("DetourTileCache/Include"))
        .file("native/recast_bridge.cpp")
        .file("native/recast_bake.cpp")
        .file("native/detour_query.cpp")
        .file("native/detour_off_mesh_connections.cpp")
        .file("native/detour_crowd.cpp")
        .file("native/detour_tile_cache.cpp")
        .file("native/detour_tile_cache_raster.cpp");

    for directory in [
        vendor.join("Recast/Source"),
        vendor.join("Detour/Source"),
        vendor.join("DetourCrowd/Source"),
        vendor.join("DetourTileCache/Source"),
    ] {
        add_cpp_sources(&mut build, &directory);
    }

    // `cc` can fail to identify a bare MSVC invocation when the build is
    // launched outside a VS developer shell.  In that case
    // `get_compiler().is_like_msvc()` reports false and `flag_if_supported`
    // silently drops the standard-mode flag, leaving `std::clamp` unavailable.
    // The Rust build target is authoritative here, so force the appropriate
    // spelling instead of relying on compiler-family probing.
    if cfg!(target_env = "msvc") {
        build.flag("/std:c++17");
        // Bridge sources and headers contain UTF-8 comments. An ambient
        // Windows code page must not change how MSVC parses these inputs.
        build.flag("/utf-8");
    } else {
        build.flag("-std=c++17");
    }

    build.compile("zircon_navigation_recast_bridge");

    println!("cargo:rerun-if-changed=native/recast_bridge.cpp");
    println!("cargo:rerun-if-changed=native/recast_bridge.h");
    println!("cargo:rerun-if-changed=native/recast_bake.cpp");
    println!("cargo:rerun-if-changed=native/detour_query.cpp");
    println!("cargo:rerun-if-changed=native/detour_off_mesh_connections.cpp");
    println!("cargo:rerun-if-changed=native/detour_off_mesh_connections.h");
    println!("cargo:rerun-if-changed=native/detour_crowd.cpp");
    println!("cargo:rerun-if-changed=native/detour_tile_cache.cpp");
    println!("cargo:rerun-if-changed=native/detour_tile_cache_raster.cpp");
    println!("cargo:rerun-if-changed=native/detour_tile_cache_raster.h");
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
