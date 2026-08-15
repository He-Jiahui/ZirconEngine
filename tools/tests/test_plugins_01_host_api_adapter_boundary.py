from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ADAPTER_ROOT = REPO_ROOT / "zircon_runtime/src/plugin/native_plugin_loader"
ADAPTER_PATH = ADAPTER_ROOT / "host_api_adapter.rs"
ABI_DECODE_ROOT = ADAPTER_ROOT / "host_api_adapter/abi_decode"
CONTEXT_HANDLES_ROOT = ADAPTER_ROOT / "host_api_adapter/context_handles"
BRIDGE_SCOPE_ROOT = ADAPTER_ROOT / "host_api_adapter/bridge_scope"
REGISTRATION_POLICY_ROOT = ADAPTER_ROOT / "host_api_adapter/registration_policy"
ECS_REGISTRATION_ROOT = ADAPTER_ROOT / "host_api_adapter/ecs_registration"
LIVE_HOST_ROOT = ADAPTER_ROOT / "native_plugin_live_host"
PUBLIC_FN_PREFIX = (
    r"pub\s+(?:(?:const|async|unsafe|extern(?:\s+\"[^\"]+\")?)\s+)*fn\s+"
)


def _rust_code_shape(source: str) -> str:
    shaped = list(source)
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = len(source) if end < 0 else end
            shaped[index:end] = " " * (end - index)
            index = end
            continue
        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            shaped[index:end] = " " * (end - index)
            index = end
            continue

        raw = re.match(r'(?:br|r)(?P<hashes>#{0,255})"', source[index:])
        if raw:
            delimiter = '"' + raw.group("hashes")
            content_start = index + raw.end()
            end = source.find(delimiter, content_start)
            end = len(source) if end < 0 else end + len(delimiter)
            shaped[index:end] = " " * (end - index)
            index = end
            continue

        if source[index] == '"':
            end = index + 1
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            shaped[index:end] = " " * (end - index)
            index = end
            continue

        if source[index] == "'" and index + 2 < len(source):
            char_end = index + (3 if source[index + 1] == "\\" else 2)
            if char_end < len(source) and source[char_end] == "'":
                char_end += 1
                shaped[index:char_end] = " " * (char_end - index)
                index = char_end
                continue
        index += 1
    return "".join(shaped)


def _public_methods_in_impl(source: str, type_name: str) -> set[str]:
    shaped = _rust_code_shape(source)
    methods: set[str] = set()
    impl_pattern = re.compile(rf"\bimpl\s+{re.escape(type_name)}\s*\{{")
    for implementation in impl_pattern.finditer(shaped):
        body_start = implementation.end()
        depth = 1
        cursor = body_start
        while cursor < len(shaped) and depth:
            if shaped[cursor] == "{":
                depth += 1
            elif shaped[cursor] == "}":
                depth -= 1
            cursor += 1
        if depth:
            raise AssertionError(f"unterminated impl {type_name}")
        body = shaped[body_start : cursor - 1]
        methods.update(
            re.findall(
                rf"\b{PUBLIC_FN_PREFIX}([A-Za-z_][A-Za-z0-9_]*)\s*\(",
                body,
            )
        )
    return methods


def _production_rust_sources(root: Path) -> str:
    paths = [
        path
        for path in root.rglob("*.rs")
        if "tests" not in path.parts and path.name != "tests.rs"
    ]
    return "\n".join(path.read_text(encoding="utf-8") for path in paths)


class Plugins01HostApiAdapterBoundaryTests(unittest.TestCase):
    def test_impl_method_inventory_finds_every_public_method(self) -> None:
        source = """
impl ExampleBackend {
    pub fn first(&self) {}
    pub(crate) fn internal(&self) {}
}

impl ExampleBackend {
    pub async fn second(&self) {
        let _nested = Some(());
        let _diagnostic = "pub fn phantom() { a brace is not structure: }";
        let _raw = r#"nor is { this"#;
    }

    // pub fn commented_out(&self) {}
    pub unsafe extern "C" fn third(&self) {}
}
"""

        self.assertEqual(
            _public_methods_in_impl(source, "ExampleBackend"),
            {"first", "second", "third"},
        )

    def test_abi_decode_is_a_private_folder_backed_boundary(self) -> None:
        source = ADAPTER_PATH.read_text(encoding="utf-8")

        self.assertIn("mod abi_decode;", source)
        for name in ("mod.rs", "error.rs", "read.rs", "system.rs"):
            self.assertTrue((ABI_DECODE_ROOT / name).is_file(), name)

    def test_root_adapter_keeps_abi_decode_implementation_out_of_ffi_wiring(self) -> None:
        source = ADAPTER_PATH.read_text(encoding="utf-8")

        for implementation in (
            "enum AbiDecodeError",
            "fn stage_from_abi(",
            "unsafe fn read_byte_slices(",
            "unsafe fn read_v4_byte_slices(",
            "unsafe fn read_v4_system_accesses(",
            "fn v4_thread_affinity_from_abi(",
            "unsafe fn read_utf8(",
        ):
            self.assertNotIn(implementation, source)

        error_source = (ABI_DECODE_ROOT / "error.rs").read_text(encoding="utf-8")
        self.assertIn("AbiDecode {", error_source)
        self.assertIn("source: AbiDecodeError", error_source)

    def test_adapter_error_preserves_the_decoded_error_source_contract(self) -> None:
        source = (ABI_DECODE_ROOT / "error.rs").read_text(encoding="utf-8")

        self.assertIn(
            "Self::AbiDecode { source } => std::error::Error::source(source),",
            source,
        )

    def test_context_handles_owns_generation_and_registration_lifetime_state(self) -> None:
        root = ADAPTER_PATH.read_text(encoding="utf-8")
        bridge_scope = (BRIDGE_SCOPE_ROOT / "mod.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("mod context_handles;", root)
        for name in ("mod.rs", "registry.rs"):
            self.assertTrue((CONTEXT_HANDLES_ROOT / name).is_file(), name)

        for implementation in (
            "pub(super) struct HostContextRegistry",
            "pub(super) struct NativeHostRegistrationScopeState",
            "pub(super) enum NativeHostApiV3Context",
            "fn encode_handle(",
        ):
            self.assertNotIn(implementation, bridge_scope)

    def test_bridge_scope_owns_bridge_context_dispatch_and_method_storage(self) -> None:
        root = ADAPTER_PATH.read_text(encoding="utf-8")
        source = (BRIDGE_SCOPE_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("mod bridge_scope;", root)
        self.assertIn("pub use bridge_scope::NativeHostBridgeCallScope;", root)
        self.assertFalse((ADAPTER_ROOT / "host_api_adapter/context_registry.rs").exists())
        for implementation in (
            "pub struct NativeHostBridgeCallScope",
            "pub(super) struct NativeHostBridgeCallContext",
            "pub(super) struct DenseBridgeMethodTable",
            'pub(super) unsafe extern "C" fn native_host_bridge_call_v1(',
            "pub(super) fn bridge_context_for(",
        ):
            self.assertIn(implementation, source)
            self.assertNotIn(implementation, root)

        self.assertIn("library_owner: Option<NativePluginLibraryGenerationOwner>", source)
        self.assertNotIn("NativeHostApiV3RegistrationContext", source)

    def test_registration_policy_assembles_v4_scope_without_registry_mutation(self) -> None:
        root = ADAPTER_PATH.read_text(encoding="utf-8")
        source = (REGISTRATION_POLICY_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("mod registration_policy;", root)
        self.assertIn("pub use registration_policy::{", root)
        self.assertIn("pub struct NativeHostApiV4RegistrationPolicy", source)
        self.assertIn("pub struct NativeHostApiV4RegistrationScope", source)
        self.assertIn("pub fn api(&self) -> ZrHostApiV4", source)
        for mutation in (
            ".register_external_native_system(",
            ".register_component(",
            ".register_native_system::<",
        ):
            self.assertNotIn(mutation, source)

    def test_ecs_registration_is_the_only_adapter_registry_mutation_boundary(self) -> None:
        root = ADAPTER_PATH.read_text(encoding="utf-8")
        source = (ECS_REGISTRATION_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("mod ecs_registration;", root)
        for implementation in (
            'pub(super) unsafe extern "C" fn native_host_register_system_v2(',
            'pub(super) unsafe extern "C" fn native_host_register_component_v1(',
            "unsafe fn register_system_from_abi_v4(",
            "unsafe fn register_component_from_abi(",
            "pub(super) struct NativeDynamicAccess",
        ):
            self.assertIn(implementation, source)

        for mutation in (
            ".register_external_native_system(",
            ".register_component(",
            ".register_native_system::<",
        ):
            self.assertIn(mutation, source)
            for other in (ABI_DECODE_ROOT, BRIDGE_SCOPE_ROOT, CONTEXT_HANDLES_ROOT, REGISTRATION_POLICY_ROOT):
                for path in other.glob("*.rs"):
                    if path.name == "tests.rs":
                        continue
                    self.assertNotIn(mutation, path.read_text(encoding="utf-8"), path)

    def test_root_adapter_is_only_private_wiring_and_narrow_reexports(self) -> None:
        source = ADAPTER_PATH.read_text(encoding="utf-8")

        self.assertLessEqual(len(source.splitlines()), 32)
        for module in (
            "abi_decode",
            "bridge_scope",
            "context_handles",
            "ecs_registration",
            "registration_policy",
        ):
            self.assertIn(f"mod {module};", source)

        for implementation in (
            "pub struct NativeHostApiV3RegistrationScope",
            "pub struct NativeHostApiV4RegistrationScope",
            "pub struct NativeHostApiV4RegistrationPolicy",
            'unsafe extern "C" fn',
            "RuntimeExtensionRegistry",
        ):
            self.assertNotIn(implementation, source)

    def test_internal_visibility_matches_the_moved_module_owners(self) -> None:
        abi_error = (ABI_DECODE_ROOT / "error.rs").read_text(encoding="utf-8")
        abi_read = (ABI_DECODE_ROOT / "read.rs").read_text(encoding="utf-8")
        abi_system = (ABI_DECODE_ROOT / "system.rs").read_text(encoding="utf-8")
        contexts = (CONTEXT_HANDLES_ROOT / "registry.rs").read_text(encoding="utf-8")
        bridge = (BRIDGE_SCOPE_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("pub(in super::super) enum AbiDecodeError", abi_error)
        self.assertIn("pub(in super::super) enum NativeHostApiAdapterError", abi_error)
        self.assertIn("pub(in super::super) unsafe fn read_utf8", abi_read)
        self.assertIn("pub(in super::super) fn stage_from_abi", abi_system)
        self.assertIn("pub(in super::super) enum NativeHostApiV3Context", contexts)
        self.assertIn("pub(in super::super) fn close_and_wait", contexts)
        for constructor in (
            "with_methods_and_owner",
            "from_method_descriptors_with_owner",
            "from_method_descriptor_refs_with_owner",
        ):
            self.assertIn(f"pub(in super::super) fn {constructor}", bridge)

        for domain in (
            ABI_DECODE_ROOT,
            BRIDGE_SCOPE_ROOT,
            CONTEXT_HANDLES_ROOT,
            ECS_REGISTRATION_ROOT,
            REGISTRATION_POLICY_ROOT,
        ):
            for path in domain.glob("*.rs"):
                self.assertNotIn("pub(crate)", path.read_text(encoding="utf-8"), path)

    def test_behavior_tests_follow_their_owning_adapter_domains(self) -> None:
        self.assertFalse((ADAPTER_ROOT / "host_api_adapter/tests.rs").exists())
        for domain in (
            ABI_DECODE_ROOT,
            BRIDGE_SCOPE_ROOT,
            CONTEXT_HANDLES_ROOT,
            ECS_REGISTRATION_ROOT,
            REGISTRATION_POLICY_ROOT,
        ):
            module_source = (domain / "mod.rs").read_text(encoding="utf-8")
            self.assertTrue((domain / "tests.rs").is_file(), domain)
            self.assertIn("#[cfg(test)]", module_source, domain)
            self.assertIn("mod tests;", module_source, domain)

    def test_app_and_editor_production_use_native_plugin_host_handle_boundary(self) -> None:
        production_roots = [
            REPO_ROOT / "zircon_app" / "src",
            REPO_ROOT / "zircon_editor" / "src",
        ]
        production_sources = [
            source
            for root in production_roots
            for source in root.rglob("*.rs")
            if "tests" not in source.parts and "test" not in source.name
        ]
        offenders = []
        for source in production_sources:
            text = source.read_text(encoding="utf-8")
            for forbidden in ("NativePluginLiveHost", "NativePluginLoader"):
                if forbidden in text:
                    offenders.append(
                        f"{source.relative_to(REPO_ROOT).as_posix()}: {forbidden}"
                    )

        self.assertEqual(
            offenders,
            [],
            "Editor production code must depend on the typed native plugin host handle and "
            "runtime-owned discovery functions, not concrete loader/live-host backends",
        )

        public_native_surface = (
            REPO_ROOT / "zircon_runtime/src/plugin/native.rs"
        ).read_text(encoding="utf-8")
        for forbidden in ("NativePluginLiveHost", "NativePluginLoader"):
            self.assertIsNone(
                re.search(rf"\b{forbidden}\b", public_native_surface),
                forbidden,
            )

    def test_typed_host_boundary_preserves_concrete_backend_capabilities(self) -> None:
        handle_source = (
            ADAPTER_ROOT / "native_plugin_host_handle.rs"
        ).read_text(encoding="utf-8")
        handle_code = _rust_code_shape(handle_source)
        loader_surface = (ADAPTER_ROOT / "mod.rs").read_text(encoding="utf-8")
        public_native_surface = (
            REPO_ROOT / "zircon_runtime/src/plugin/native.rs"
        ).read_text(encoding="utf-8")

        live_host_source = (ADAPTER_ROOT / "native_plugin_live_host.rs").read_text(
            encoding="utf-8"
        ) + _production_rust_sources(LIVE_HOST_ROOT)
        backend_host_methods = _public_methods_in_impl(
            live_host_source, "NativePluginLiveHost"
        )
        handle_methods = _public_methods_in_impl(
            handle_code, "NativePluginHostHandle"
        )
        self.assertEqual(handle_methods - {"downgrade"}, backend_host_methods)
        for method in backend_host_methods:
            self.assertRegex(
                handle_code,
                rf"{PUBLIC_FN_PREFIX}{re.escape(method)}\b"
                rf"(?:(?!\n\s*{PUBLIC_FN_PREFIX})[\s\S])*?self\.backend\s*\.\s*"
                rf"{re.escape(method)}\s*\(",
                f"typed host method {method} must delegate to the same backend method",
            )

        loader_projection = {
            "discover": "discover_native_plugins",
            "refresh_discovery_manifest": "refresh_native_plugin_discovery_manifest",
            "remove_discovered_path": "remove_discovered_native_plugin_path",
            "discovery_generation": "native_plugin_discovery_generation",
            "discover_from_load_manifest": "discover_native_plugins_from_load_manifest",
            "load_discovered_all": "load_discovered_native_plugins",
            "load_discovered_runtime": "load_discovered_native_runtime_plugins",
            "load_discovered_editor": "load_discovered_native_editor_plugins",
            "load_all_from_load_manifest": "load_native_plugins_from_load_manifest",
            "load_runtime_from_load_manifest": "load_native_runtime_from_load_manifest",
            "load_editor_from_load_manifest": "load_native_editor_from_load_manifest",
        }
        backend_loader_methods = _public_methods_in_impl(
            _production_rust_sources(ADAPTER_ROOT), "NativePluginLoader"
        )
        self.assertEqual(set(loader_projection), backend_loader_methods)
        for backend_method, function in loader_projection.items():
            self.assertIn(f"pub fn {function}(", handle_code, function)
            self.assertRegex(
                handle_code,
                rf"{PUBLIC_FN_PREFIX}{re.escape(function)}\b"
                rf"(?:(?!\n\s*{PUBLIC_FN_PREFIX})[\s\S])*?NativePluginLoader\s*\.\s*"
                rf"{re.escape(backend_method)}\s*\(",
                f"runtime-owned loader function {function} must delegate to {backend_method}",
            )
            self.assertRegex(loader_surface, rf"\b{function}\b", function)
            self.assertRegex(public_native_surface, rf"\b{function}\b", function)

        for handle in ("NativePluginHostHandle", "NativePluginHostWeakHandle"):
            self.assertRegex(loader_surface, rf"\b{handle}\b", handle)
            self.assertRegex(public_native_surface, rf"\b{handle}\b", handle)


if __name__ == "__main__":
    unittest.main()
