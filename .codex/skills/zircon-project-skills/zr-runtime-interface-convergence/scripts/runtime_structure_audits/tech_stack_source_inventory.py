from __future__ import annotations


EXPECTED_MANIFEST_COUNT = 5
EXPECTED_NON_DEPENDENCY_COUNT = 5
EXPECTED_ZIP_DEPENDENCY_COUNT = 1
EXPECTED_EDITOR_ONLY_CANDIDATE_COUNT = 3
ZR_VM_PLUGIN_MANIFEST = "zircon_plugins/zr_vm_language/runtime/Cargo.toml"
ZR_VM_BACKEND_FEATURE = "backend-zr-vm"
ZR_VM_BINDING_DEPENDENCY_PREFIX = "zr_vm_rust_binding"
ZR_VM_EXTERNAL_PATH_PREFIX = "../../../../zr_vm/"
ZIP_DEPENDENCY_LINE = (
    'zip = { version = "9.0.0-pre2", default-features = false, '
    'features = ["deflate-flate2"] }'
)

MANIFEST_FILES = (
    "Cargo.toml",
    "zircon_runtime/Cargo.toml",
    "zircon_runtime_interface/Cargo.toml",
    "zircon_editor/Cargo.toml",
    "zircon_plugins/physics/runtime/Cargo.toml",
)
REQUIRED_VERSION_ANCHORS = (
    "0.31.0-beta.2",
    "9.0.0-rc.3",
    'wgpu = "29.0.1"',
    'naga = { version = "29.0.1"',
    'glam = { version = "0.32.1"',
    'glyphon = { version = "0.11.0", optional = true }',
    'fontsdf = { version = "0.5.3", optional = true }',
    'zstd = "0.13.3"',
    ZIP_DEPENDENCY_LINE,
)
NON_DEPENDENCIES = (
    "cosmic-text",
    "kira",
    "rfd",
    "arboard",
    "tar",
)
