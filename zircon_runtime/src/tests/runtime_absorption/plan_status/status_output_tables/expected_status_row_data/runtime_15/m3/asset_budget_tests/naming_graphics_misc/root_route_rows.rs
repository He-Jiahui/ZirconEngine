type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 naming-boundary route-owner split",
        &[
            "runtime_15_naming_boundary_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary.rs",
            "tests/runtime_absorption/naming_boundary/top_level.rs",
            "tests/runtime_absorption/naming_boundary/classifiers.rs",
            "tests/runtime_absorption/naming_boundary/lexical_scan.rs",
            "tests/runtime_absorption/naming_boundary/support.rs",
            "runtime_15_naming_boundary_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 root-surface route-owner split",
        &[
            "runtime_15_root_surface_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/root_surface.rs",
            "tests/runtime_absorption/root_surface/public_surface.rs",
            "tests/runtime_absorption/root_surface/graphics_alias.rs",
            "tests/runtime_absorption/root_surface/docs.rs",
            "tests/runtime_absorption/root_surface/inventory.rs",
            "tests/runtime_absorption/root_surface/split_layout.rs",
            "runtime_15_root_surface_route_owner_is_folder_backed",
        ],
    ),
];
