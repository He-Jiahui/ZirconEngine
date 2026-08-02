from __future__ import annotations

import re
import sys
import unittest
from pathlib import Path

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_MANIFEST = REPO_ROOT / "zircon_runtime" / "Cargo.toml"
APP_MANIFEST = REPO_ROOT / "zircon_app" / "Cargo.toml"
FRAMEWORK_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "mod.rs"
FRAMEWORK_TESTS = FRAMEWORK_ROOT.parent / "tests.rs"
MANAGER_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "core" / "manager"
AI_PLUGIN_MANIFEST = REPO_ROOT / "zircon_plugins" / "ai" / "runtime" / "Cargo.toml"
ASSET_RUNTIME_PATH = REPO_ROOT / "zircon_runtime" / "src" / "asset" / "runtime_asset_path.rs"
ASSET_RUNTIME_PATH_DIAGNOSTICS = ASSET_RUNTIME_PATH.parent / "runtime_asset_path"
NET_DOWNLOAD_CONTRACT = (
    REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "net" / "download.rs"
)
NET_CONTRACT_ROOT = NET_DOWNLOAD_CONTRACT.parent / "mod.rs"
ASSET_PACK_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "asset" / "pack"
EXPORT_PACK_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "bin" / "zircon_export_pack"
NET_PLUGIN_MANIFESTS = (
    REPO_ROOT / "zircon_plugins" / "net" / "runtime" / "Cargo.toml",
    REPO_ROOT
    / "zircon_plugins"
    / "net"
    / "features"
    / "content_download"
    / "runtime"
    / "Cargo.toml",
    REPO_ROOT / "zircon_plugins" / "net" / "features" / "http" / "runtime" / "Cargo.toml",
    REPO_ROOT
    / "zircon_plugins"
    / "net"
    / "features"
    / "reliable_udp"
    / "runtime"
    / "Cargo.toml",
    REPO_ROOT
    / "zircon_plugins"
    / "net"
    / "features"
    / "replication"
    / "runtime"
    / "Cargo.toml",
    REPO_ROOT / "zircon_plugins" / "net" / "features" / "rpc" / "runtime" / "Cargo.toml",
    REPO_ROOT
    / "zircon_plugins"
    / "net"
    / "features"
    / "websocket"
    / "runtime"
    / "Cargo.toml",
)
SOUND_PLUGIN_MANIFESTS = (
    REPO_ROOT / "zircon_plugins" / "sound" / "runtime" / "Cargo.toml",
    REPO_ROOT / "zircon_plugins" / "sound" / "editor" / "Cargo.toml",
)
PHYSICS_PLUGIN_MANIFEST = (
    REPO_ROOT / "zircon_plugins" / "physics" / "runtime" / "Cargo.toml"
)
AUDIO_CONTRACT_ROOT = (
    REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "audio"
)
SOUND_CONTRACT_ROOT = (
    REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "sound"
)
PHYSICS_CONTRACT_ROOT = (
    REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "physics"
)
SCENE_PHYSICS_ROOT = (
    REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "scene" / "physics"
)
CORE_RUNTIME_DIAGNOSTICS_ROOT = (
    REPO_ROOT / "zircon_runtime" / "src" / "core" / "runtime" / "diagnostics"
)
RUNTIME_DIAGNOSTICS_FACADE_ROOT = (
    REPO_ROOT / "zircon_runtime" / "src" / "runtime_diagnostics"
)
LEVEL_SYSTEM_SOURCE = REPO_ROOT / "zircon_runtime" / "src" / "scene" / "level_system.rs"
GRAPHICS_RICH_TEXT_RENDER_SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "graphics"
    / "scene"
    / "scene_renderer"
    / "ui"
    / "render"
    / "rich_text.rs"
)
GRAPHICS_UI_TEXTURE_SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "graphics"
    / "scene"
    / "resources"
    / "ui_texture.rs"
)
GRAPHICS_TYPES_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "graphics" / "types"

PERSISTENT_PHYSICS_TYPES = {
    "combine_rule.rs": "pub enum PhysicsCombineRule",
    "joint_constraint_metadata.rs": "pub struct PhysicsJointConstraintMetadata",
    "joint_drive.rs": "pub struct PhysicsJointDrive",
    "material_metadata.rs": "pub struct PhysicsMaterialMetadata",
    "skeleton_joint_binding.rs": "pub struct PhysicsSkeletonJointBinding",
}


def references_optional_physics_owner(source: str, type_name: str) -> bool:
    escaped_type = re.escape(type_name)
    optional_owner_path = re.compile(
        rf"\bcore\s*::\s*framework\s*::\s*physics\s*::\s*"
        rf"(?:\{{[^;]*\b{escaped_type}\b[^;]*\}}|\b{escaped_type}\b)",
        re.DOTALL,
    )
    return optional_owner_path.search(source) is not None


class Frameworks03ContractFeatureBoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.runtime_manifest = tomllib.loads(RUNTIME_MANIFEST.read_text(encoding="utf-8"))
        cls.app_manifest = tomllib.loads(APP_MANIFEST.read_text(encoding="utf-8"))
        cls.framework_root = FRAMEWORK_ROOT.read_text(encoding="utf-8")
        cls.framework_tests = FRAMEWORK_TESTS.read_text(encoding="utf-8")
        cls.manager_root = (MANAGER_ROOT / "mod.rs").read_text(encoding="utf-8")
        cls.manager_resolver = (MANAGER_ROOT / "resolver.rs").read_text(encoding="utf-8")
        cls.manager_names = (MANAGER_ROOT / "service_names.rs").read_text(
            encoding="utf-8"
        )
        cls.ai_plugin_manifest = tomllib.loads(
            AI_PLUGIN_MANIFEST.read_text(encoding="utf-8")
        )
        cls.asset_runtime_path = ASSET_RUNTIME_PATH.read_text(encoding="utf-8")

    def test_ai_contract_feature_is_forwarded_by_client_presets(self) -> None:
        runtime_features = self.runtime_manifest["features"]
        app_features = self.app_manifest["features"]

        self.assertEqual(runtime_features["ai-contracts"], [])
        self.assertEqual(app_features["ai-contracts"], ["zircon_runtime/ai-contracts"])
        for preset in ("target-client", "target-editor-host"):
            self.assertIn("ai-contracts", runtime_features[preset], preset)
            self.assertIn("ai-contracts", app_features[preset], preset)
        self.assertNotIn("ai-contracts", runtime_features["target-server"])
        self.assertNotIn("ai-contracts", app_features["target-server"])

    def test_ai_contract_declarations_are_feature_gated(self) -> None:
        self.assertIn(
            '#[cfg(feature = "ai-contracts")]\npub mod ai;', self.framework_root
        )
        self.assertNotIn(
            '#[cfg(feature = "ai-contracts")]\npub use crate::core::framework::ai::AiManager;',
            self.manager_root,
        )
        self.assertIn(
            '#[cfg(feature = "ai-contracts")]\nuse crate::core::framework::ai::AiManager;',
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "ai-contracts")]\n'
            "define_manager_handle_access!(AiManager, ai_manager_handle, "
            "AI_MANAGER_NAME, ai_handle);",
            self.manager_resolver,
        )
        self.assertNotIn(
            '#[cfg(feature = "ai-contracts")]\n'
            "define_manager_handle_access!(\n"
            "    RenderingManager,",
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "ai-contracts")]\npub const AI_MANAGER_NAME:',
            self.manager_names,
        )

    def test_ai_plugin_explicitly_requests_its_contract(self) -> None:
        runtime_dependency = self.ai_plugin_manifest["dependencies"]["zircon_runtime"]
        self.assertIn("ai-contracts", runtime_dependency.get("features", []))

    def test_foundational_asset_path_uses_a_compile_time_diagnostics_adapter(self) -> None:
        self.assertNotIn("crate::diagnostic_log", self.asset_runtime_path)
        self.assertIn(
            '#[cfg(feature = "diagnostic-log")]\n'
            '#[path = "runtime_asset_path/diagnostics_enabled.rs"]\n'
            "mod diagnostics;",
            self.asset_runtime_path,
        )
        self.assertIn(
            '#[cfg(not(feature = "diagnostic-log"))]\n'
            '#[path = "runtime_asset_path/diagnostics_disabled.rs"]\n'
            "mod diagnostics;",
            self.asset_runtime_path,
        )

        enabled = (
            ASSET_RUNTIME_PATH_DIAGNOSTICS / "diagnostics_enabled.rs"
        ).read_text(encoding="utf-8")
        disabled = (
            ASSET_RUNTIME_PATH_DIAGNOSTICS / "diagnostics_disabled.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("crate::diagnostic_log", enabled)
        self.assertNotIn("crate::diagnostic_log", disabled)

    def test_zrpack_protocol_types_have_one_asset_pack_owner(self) -> None:
        asset_manifest = (ASSET_PACK_ROOT / "manifest.rs").read_text(encoding="utf-8")
        net_download = NET_DOWNLOAD_CONTRACT.read_text(encoding="utf-8")
        net_root = NET_CONTRACT_ROOT.read_text(encoding="utf-8")
        export_main = (EXPORT_PACK_ROOT / "main.rs").read_text(encoding="utf-8")

        for declaration in ("pub struct ZrPackManifest", "pub struct ZrChunkEntry"):
            self.assertIn(declaration, asset_manifest)
            self.assertNotIn(declaration, net_download)
        self.assertNotIn("ZrChunkEntry", net_root)
        self.assertNotIn("ZrPackManifest", net_root)
        self.assertNotIn("mod core;", export_main)
        self.assertFalse((EXPORT_PACK_ROOT / "core.rs").exists())

        for source in ASSET_PACK_ROOT.glob("*.rs"):
            self.assertNotIn(
                "core::framework::net",
                source.read_text(encoding="utf-8"),
                source.name,
            )

    def test_net_contract_feature_is_forwarded_by_client_presets(self) -> None:
        runtime_features = self.runtime_manifest["features"]
        app_features = self.app_manifest["features"]

        self.assertEqual(runtime_features["net-contracts"], [])
        self.assertEqual(app_features["net-contracts"], ["zircon_runtime/net-contracts"])
        for preset in ("target-client", "target-editor-host"):
            self.assertIn("net-contracts", runtime_features[preset], preset)
            self.assertIn("net-contracts", app_features[preset], preset)
        self.assertNotIn("net-contracts", runtime_features["target-server"])
        self.assertNotIn("net-contracts", app_features["target-server"])

    def test_net_contract_declarations_are_feature_gated(self) -> None:
        self.assertIn(
            '#[cfg(feature = "net-contracts")]\npub mod net;', self.framework_root
        )
        self.assertNotIn(
            '#[cfg(feature = "net-contracts")]\npub use crate::core::framework::net::NetManager;',
            self.manager_root,
        )
        self.assertIn(
            '#[cfg(feature = "net-contracts")]\nuse crate::core::framework::net::NetManager;',
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "net-contracts")]\n'
            "define_manager_handle_access!(NetManager, net_manager_handle, "
            "NET_MANAGER_NAME, net_handle);",
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "net-contracts")]\npub const NET_MANAGER_NAME:',
            self.manager_names,
        )

    def test_net_plugins_explicitly_request_the_contract(self) -> None:
        for manifest_path in NET_PLUGIN_MANIFESTS:
            manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
            runtime_dependency = manifest["dependencies"]["zircon_runtime"]
            self.assertIn(
                "net-contracts",
                runtime_dependency.get("features", []),
                manifest_path.relative_to(REPO_ROOT).as_posix(),
            )

    def test_framework_contract_tests_gate_optional_net_fixtures(self) -> None:
        self.assertIn(
            '#[cfg(feature = "net-contracts")]\n'
            "use super::net::{NetEndpoint, NetError, NetPacket, NetSocketId};",
            self.framework_tests,
        )
        self.assertIn(
            '#[cfg(feature = "net-contracts")]\n'
            "    let socket = NetSocketId::new(5);",
            self.framework_tests,
        )
        self.assertIn(
            '#[cfg(feature = "net-contracts")]\n'
            "    {\n"
            "        assert_eq!(socket.raw(), 5);",
            self.framework_tests,
        )

    def test_sound_contract_feature_is_forwarded_by_client_presets(self) -> None:
        runtime_features = self.runtime_manifest["features"]
        app_features = self.app_manifest["features"]

        self.assertEqual(runtime_features.get("sound-contracts"), [])
        self.assertEqual(
            app_features.get("sound-contracts"), ["zircon_runtime/sound-contracts"]
        )
        for preset in ("target-client", "target-editor-host"):
            self.assertIn("sound-contracts", runtime_features[preset], preset)
            self.assertIn("sound-contracts", app_features[preset], preset)
        self.assertNotIn("sound-contracts", runtime_features["target-server"])
        self.assertNotIn("sound-contracts", app_features["target-server"])

    def test_sound_contract_declarations_are_feature_gated(self) -> None:
        self.assertIn(
            '#[cfg(feature = "sound-contracts")]\npub mod sound;',
            self.framework_root,
        )
        self.assertNotIn(
            '#[cfg(feature = "sound-contracts")]\n'
            "pub use crate::core::framework::sound::SoundManager;",
            self.manager_root,
        )
        self.assertIn(
            '#[cfg(feature = "sound-contracts")]\n'
            "use crate::core::framework::sound::SoundManager;",
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "sound-contracts")]\n'
            "define_manager_handle_access!(\n"
            "    SoundManager,",
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "sound-contracts")]\npub const SOUND_MANAGER_NAME:',
            self.manager_names,
        )

    def test_sound_plugins_explicitly_request_the_contract(self) -> None:
        for manifest_path in SOUND_PLUGIN_MANIFESTS:
            manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
            runtime_dependency = manifest["dependencies"]["zircon_runtime"]
            self.assertIn(
                "sound-contracts",
                runtime_dependency.get("features", []),
                manifest_path.relative_to(REPO_ROOT).as_posix(),
            )

    def test_audio_channel_topology_has_one_neutral_owner(self) -> None:
        self.assertTrue((AUDIO_CONTRACT_ROOT / "mod.rs").is_file())
        self.assertTrue((AUDIO_CONTRACT_ROOT / "channel_layout.rs").is_file())
        audio_root = (AUDIO_CONTRACT_ROOT / "mod.rs").read_text(encoding="utf-8")
        audio_layout = (AUDIO_CONTRACT_ROOT / "channel_layout.rs").read_text(
            encoding="utf-8"
        )
        sound_root = (SOUND_CONTRACT_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("pub struct AudioChannelLayout", audio_layout)
        self.assertIn("pub enum AudioSpeakerChannel", audio_layout)
        self.assertIn(
            "pub use channel_layout::{AudioChannelLayout, AudioSpeakerChannel};",
            audio_root,
        )
        self.assertNotIn("channel_layout", sound_root)
        self.assertFalse((SOUND_CONTRACT_ROOT / "channel_layout.rs").exists())

        for root in ("zircon_runtime", "zircon_plugins", "zircon_editor", "zircon_app"):
            for source in (REPO_ROOT / root).rglob("*.rs"):
                content = source.read_text(encoding="utf-8")
                self.assertNotIn("SoundChannelLayout", content, source)
                self.assertNotIn("SoundSpeakerChannel", content, source)

    def test_physics_contract_feature_is_forwarded_by_client_presets(self) -> None:
        runtime_features = self.runtime_manifest["features"]
        app_features = self.app_manifest["features"]

        self.assertEqual(runtime_features.get("physics-contracts"), [])
        self.assertEqual(
            app_features.get("physics-contracts"), ["zircon_runtime/physics-contracts"]
        )
        for preset in ("target-client", "target-editor-host"):
            self.assertIn("physics-contracts", runtime_features[preset], preset)
            self.assertIn("physics-contracts", app_features[preset], preset)
        self.assertNotIn("physics-contracts", runtime_features["target-server"])
        self.assertNotIn("physics-contracts", app_features["target-server"])

    def test_physics_contract_declarations_are_feature_gated(self) -> None:
        self.assertIn(
            '#[cfg(feature = "physics-contracts")]\npub mod physics;',
            self.framework_root,
        )
        self.assertNotIn(
            '#[cfg(feature = "physics-contracts")]\n'
            "pub use crate::core::framework::physics::PhysicsManager;",
            self.manager_root,
        )
        self.assertIn(
            '#[cfg(feature = "physics-contracts")]\n'
            "use crate::core::framework::physics::PhysicsManager;",
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "physics-contracts")]\n'
            "define_manager_handle_access!(\n"
            "    PhysicsManager,",
            self.manager_resolver,
        )
        self.assertIn(
            '#[cfg(feature = "physics-contracts")]\npub const PHYSICS_MANAGER_NAME:',
            self.manager_names,
        )

    def test_physics_plugin_explicitly_requests_the_contract(self) -> None:
        manifest = tomllib.loads(PHYSICS_PLUGIN_MANIFEST.read_text(encoding="utf-8"))
        runtime_dependency = manifest["dependencies"]["zircon_runtime"]
        self.assertIn(
            "physics-contracts",
            runtime_dependency.get("features", []),
        )

    def test_persistent_physics_schema_has_one_always_on_scene_owner(self) -> None:
        self.assertTrue((SCENE_PHYSICS_ROOT / "mod.rs").is_file())
        scene_root = (SCENE_PHYSICS_ROOT.parent / "mod.rs").read_text(encoding="utf-8")
        scene_physics_root = (SCENE_PHYSICS_ROOT / "mod.rs").read_text(
            encoding="utf-8"
        )
        optional_physics_root = (PHYSICS_CONTRACT_ROOT / "mod.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub mod physics;", scene_root)
        for file_name, declaration in PERSISTENT_PHYSICS_TYPES.items():
            self.assertIn(
                declaration,
                (SCENE_PHYSICS_ROOT / file_name).read_text(encoding="utf-8"),
                file_name,
            )
            self.assertIn(file_name.removesuffix(".rs"), scene_physics_root)
            self.assertFalse((PHYSICS_CONTRACT_ROOT / file_name).exists(), file_name)
            self.assertNotIn(file_name.removesuffix(".rs"), optional_physics_root)

        joint_constraint = (
            SCENE_PHYSICS_ROOT / "joint_constraint_metadata.rs"
        ).read_text(encoding="utf-8")
        self.assertTrue(
            (SCENE_PHYSICS_ROOT / "joint_constraint_serde.rs").is_file()
        )
        self.assertIn("mod joint_constraint_serde;", scene_physics_root)
        self.assertNotIn("struct AxisLimitsVisitor", joint_constraint)
        self.assertNotIn("fn serialize_axis_limits", joint_constraint)

        for root in ("zircon_runtime", "zircon_plugins", "zircon_editor", "zircon_app"):
            for source in (REPO_ROOT / root).rglob("*.rs"):
                content = source.read_text(encoding="utf-8")
                if "core::framework::physics" not in content:
                    continue
                for declaration in PERSISTENT_PHYSICS_TYPES.values():
                    type_name = declaration.rsplit(" ", maxsplit=1)[-1]
                    self.assertFalse(
                        references_optional_physics_owner(content, type_name),
                        source,
                    )

    def test_persistent_physics_owner_guard_allows_scene_consumers(self) -> None:
        legal_mixed_imports = """
use zircon_runtime::core::framework::physics::{PhysicsJointSyncState, PhysicsJointType};
use zircon_runtime::core::framework::scene::physics::PhysicsJointDrive;
"""
        stale_direct_import = (
            "use zircon_runtime::core::framework::physics::PhysicsJointDrive;"
        )
        stale_grouped_import = """
use zircon_runtime::core::framework::physics::{
    PhysicsJointDrive,
    PhysicsJointSyncState,
};
"""

        self.assertFalse(
            references_optional_physics_owner(
                legal_mixed_imports,
                "PhysicsJointDrive",
            )
        )
        self.assertTrue(
            references_optional_physics_owner(
                stale_direct_import,
                "PhysicsJointDrive",
            )
        )
        self.assertTrue(
            references_optional_physics_owner(
                stale_grouped_import,
                "PhysicsJointDrive",
            )
        )

    def test_optional_physics_runtime_state_uses_declaration_adapters(self) -> None:
        level_system = LEVEL_SYSTEM_SOURCE.read_text(encoding="utf-8")
        self.assertNotIn("core::framework::physics", level_system)
        self.assertIn(
            '#[cfg(feature = "physics-contracts")]\n'
            '#[path = "level_system/physics_runtime_enabled.rs"]\n'
            "mod physics_runtime;",
            level_system,
        )
        self.assertIn(
            '#[cfg(not(feature = "physics-contracts"))]\n'
            '#[path = "level_system/physics_runtime_disabled.rs"]\n'
            "mod physics_runtime;",
            level_system,
        )

    def test_runtime_physics_diagnostics_use_a_neutral_projection_adapter(self) -> None:
        diagnostics_root = (CORE_RUNTIME_DIAGNOSTICS_ROOT / "mod.rs").read_text(
            encoding="utf-8"
        )
        facade_root = (RUNTIME_DIAGNOSTICS_FACADE_ROOT / "mod.rs").read_text(
            encoding="utf-8"
        )
        physics_projection = (
            CORE_RUNTIME_DIAGNOSTICS_ROOT / "physics.rs"
        ).read_text(
            encoding="utf-8"
        )
        backend_projection_path = (
            CORE_RUNTIME_DIAGNOSTICS_ROOT / "physics_backend.rs"
        )
        diagnostics_collection = (
            RUNTIME_DIAGNOSTICS_FACADE_ROOT / "collect.rs"
        ).read_text(encoding="utf-8")
        enabled_collection = (
            RUNTIME_DIAGNOSTICS_FACADE_ROOT / "physics_collection_enabled.rs"
        ).read_text(encoding="utf-8")
        disabled_collection = (
            RUNTIME_DIAGNOSTICS_FACADE_ROOT / "physics_collection_disabled.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("core::framework::physics", diagnostics_root)
        self.assertNotIn("core::manager", diagnostics_root)
        self.assertNotIn("core::manager", physics_projection)
        self.assertNotIn("core::manager", disabled_collection)
        self.assertIn("core::manager", enabled_collection)
        self.assertIn(
            "super::physics_collection::collect(core)", diagnostics_collection
        )

        self.assertNotIn("core::framework::physics", physics_projection)
        self.assertTrue(backend_projection_path.is_file())
        backend_projection = backend_projection_path.read_text(encoding="utf-8")
        self.assertIn(
            "pub struct RuntimePhysicsBackendDiagnostics", backend_projection
        )
        self.assertIn(
            "pub backend_status: Option<RuntimePhysicsBackendDiagnostics>",
            physics_projection,
        )
        self.assertNotIn("resolve_physics_manager", diagnostics_collection)
        self.assertIn(
            '#[cfg(feature = "physics-contracts")]\n'
            '#[path = "physics_collection_enabled.rs"]\n'
            "mod physics_collection;",
            facade_root,
        )
        self.assertIn(
            '#[cfg(not(feature = "physics-contracts"))]\n'
            '#[path = "physics_collection_disabled.rs"]\n'
            "mod physics_collection;",
            facade_root,
        )

    def test_graphics_does_not_require_the_ui_domain(self) -> None:
        graphics_types = (GRAPHICS_TYPES_ROOT / "mod.rs").read_text(encoding="utf-8")

        for source_path in (
            GRAPHICS_RICH_TEXT_RENDER_SOURCE,
            GRAPHICS_UI_TEXTURE_SOURCE,
        ):
            self.assertNotIn(
                "crate::ui",
                source_path.read_text(encoding="utf-8"),
                source_path.relative_to(REPO_ROOT).as_posix(),
            )
        self.assertNotIn(
            "viewport_render_frame_from_public_runtime",
            graphics_types,
        )
        self.assertFalse(
            (GRAPHICS_TYPES_ROOT / "viewport_render_frame_from_public_runtime.rs").exists()
        )


if __name__ == "__main__":
    unittest.main()
