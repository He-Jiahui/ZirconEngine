from __future__ import annotations


EXPECTED_MANIFEST_COUNT = 5
EXPECTED_NON_DEPENDENCY_COUNT = 4
EXPECTED_ZIP_DEPENDENCY_COUNT = 1
EXPECTED_EDITOR_ONLY_CANDIDATE_COUNT = 3
KIRA_OWNER_MANIFEST = "zircon_plugins/sound/runtime/Cargo.toml"
KIRA_DEPENDENCY_VERSION = "0.12.2"
KIRA_DEPENDENCY_LINE = 'kira = "0.12.2"'
ZR_VM_PLUGIN_MANIFEST = "zircon_plugins/zr_vm_language/runtime/Cargo.toml"
ZR_VM_BACKEND_FEATURE = "backend-zr-vm"
ZR_VM_BINDING_DEPENDENCY_PREFIX = "zr_vm_rust_binding"
ZR_VM_GIT_URL = "https://github.com/He-Jiahui/zr_vm.git"
ZR_VM_GIT_REV = "3c1352ecb6ddfab02752d46682cc376558d5f481"
RUNTIME_POLICY_OWNER = "zircon_runtime/Cargo.toml"
ZSTD_TEXTURE_POLICY_OWNER = "zircon_plugins/texture_importer/runtime/Cargo.toml"
# Package identity, fixed version, owner feature, default features and exact feature list.
RUNTIME_DEPENDENCY_POLICIES = (
    ("zstd", "0.13.3", "native-host", True, ()),
    ("zip", "9.0.0-pre2", "native-host", False, ("deflate-flate2",)),
    ("fdsm", "0.8.0", "text", True, ()),
    ("fdsm-ttf-parser", "0.2.0", "text", True, ()),
    ("ttf-parser", "0.25.1", "text", True, ()),
)
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
    'wgpu = "=29.0.3"',
    'naga = { version = "=29.0.3"',
    'glam = { version = "0.32.1"',
    'glyphon = { version = "0.11.0", optional = true }',
)
NON_DEPENDENCIES = (
    "cosmic-text",
    "rfd",
    "arboard",
    "tar",
)
