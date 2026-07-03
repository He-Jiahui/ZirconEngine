from __future__ import annotations

from pathlib import Path


def _write_complete_native_dynamic_fixture_manifest(
    repo_root: Path,
    crate_name: str,
) -> None:
    _write_repo_root_manifest(repo_root)
    (repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml").write_text(
        "\n".join(
            [
                'id = "native_dynamic_fixture"',
                'version = "0.1.0"',
                'sdk_api_version = "0.1.0"',
                'display_name = "Native Dynamic Fixture"',
                'category = "runtime"',
                'description = "Native dynamic fixture plugin."',
                'supported_targets = ["client_runtime", "editor_host"]',
                'supported_platforms = ["windows", "linux", "macos"]',
                'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                'maturity = "experimental"',
                "",
                "[distribution]",
                'forms = ["dist"]',
                'default_packaging = ["native_dynamic"]',
                "abi_version = 3",
                'engine_compat = ">=0.1, <0.2"',
                f'dist_crate = "{crate_name}"',
                'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                "",
                "[[modules]]",
                'name = "native_dynamic_fixture.runtime"',
                'kind = "runtime"',
                f'crate_name = "{crate_name}"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
            ]
        ),
        encoding="utf-8",
    )


def _replace_manifest_line(manifest_path: Path, old: str, new: str) -> None:
    text = manifest_path.read_text(encoding="utf-8")
    if old not in text:
        raise AssertionError(f"manifest fixture is missing line: {old}")
    manifest_path.write_text(text.replace(old, new), encoding="utf-8")


def _write_plugin_workspace_members(repo_root: Path, members: list[str]) -> None:
    members_text = ", ".join(f'"{member}"' for member in members)
    (repo_root / "zircon_plugins" / "Cargo.toml").write_text(
        "\n".join(
            [
                "[workspace]",
                f"members = [{members_text}]",
                'resolver = "2"',
            ]
        ),
        encoding="utf-8",
    )


def _write_sound_feature_dist_crate(repo_root: Path, crate_name: str) -> None:
    crate_root = (
        repo_root
        / "zircon_plugins"
        / "sound"
        / "features"
        / "timeline_animation_track"
        / "dist"
    )
    crate_root.mkdir(parents=True, exist_ok=True)
    (crate_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "{crate_name}"',
                'version = "0.1.0"',
                'edition = "2021"',
                "",
                "[lib]",
                'crate-type = ["cdylib"]',
                "",
                "[features]",
                'default = ["dist"]',
                "dist = []",
                "",
                "[dependencies]",
                'zircon_plugin_sdk = { workspace = true, default-features = false, features = ["native"] }',
            ]
        ),
        encoding="utf-8",
    )


def _write_complete_sound_manifest(repo_root: Path) -> None:
    _write_repo_root_manifest(repo_root)
    manifest_path = repo_root / "zircon_plugins" / "sound" / "plugin.toml"
    manifest_path.write_text(
        "\n".join(
            [
                'id = "sound"',
                'version = "0.1.0"',
                'display_name = "Sound"',
                'sdk_api_version = "0.1.0"',
                'category = "runtime"',
                'description = "Sound runtime plugin."',
                'maturity = "beta"',
                'supported_targets = ["client_runtime", "editor_host"]',
                'supported_platforms = ["windows", "linux", "macos"]',
                'capabilities = ["runtime.plugin.sound"]',
                "",
                "[[modules]]",
                'name = "sound.runtime"',
                'kind = "runtime"',
                'crate_name = "zircon_plugin_sound_runtime"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.plugin.sound"]',
                "",
                "[[optional_features]]",
                'id = "sound.timeline_animation_track"',
                'display_name = "Sound Timeline Animation Track"',
                'owner_plugin_id = "sound"',
                'provider_package_id = "sound_timeline_animation_track"',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                "enabled_by_default = false",
                "",
                "[[optional_features.dependencies]]",
                'plugin_id = "sound"',
                'capability = "runtime.plugin.sound"',
                "primary = true",
                "",
                "[[optional_features.dependencies]]",
                'plugin_id = "animation"',
                'capability = "runtime.feature.animation.timeline_event_track"',
                "primary = false",
                "",
                "[[optional_features.modules]]",
                'name = "sound.timeline_animation_track.runtime"',
                'kind = "runtime"',
                'crate_name = "zircon_plugin_sound_timeline_animation_runtime"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                "",
                "[[optional_features.modules]]",
                'name = "sound.timeline_animation_track.dist"',
                'kind = "native"',
                'crate_name = "zircon_plugin_sound_timeline_animation_dist"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                "",
                "[optional_features.distribution]",
                'forms = ["dist"]',
                'default_packaging = ["native_dynamic"]',
                "abi_version = 3",
                'engine_compat = ">=0.1, <0.2"',
                'dist_crate = "zircon_plugin_sound_timeline_animation_dist"',
                'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                'runtime_entry = "zircon_plugin_sound_timeline_animation_runtime_entry_v3"',
            ]
        ),
        encoding="utf-8",
    )


def _write_repo_root_manifest(repo_root: Path) -> None:
    repo_root.mkdir(parents=True, exist_ok=True)
    (repo_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[workspace]",
                'members = ["zircon_plugins"]',
                'resolver = "2"',
                "",
                "[workspace.package]",
                'version = "0.1.0"',
            ]
        ),
        encoding="utf-8",
    )
