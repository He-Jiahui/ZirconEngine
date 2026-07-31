from __future__ import annotations

import re
from functools import lru_cache
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_SOURCE_ROOTS = (
    REPO_ROOT / "zircon_runtime/src",
    REPO_ROOT / "zircon_runtime/tests",
    REPO_ROOT / "zircon_app/src",
    REPO_ROOT / "zircon_app/tests",
    REPO_ROOT / "zircon_editor/src",
    REPO_ROOT / "zircon_editor/tests",
    REPO_ROOT / "zircon_plugins",
)


@lru_cache(maxsize=1)
def _workspace_rust_sources() -> tuple[tuple[str, str], ...]:
    paths = sorted(
        (
            path
            for root in RUST_SOURCE_ROOTS
            if root.exists()
            for path in root.rglob("*.rs")
        ),
        key=lambda path: path.relative_to(REPO_ROOT).as_posix(),
    )
    return tuple(
        (
            path.relative_to(REPO_ROOT).as_posix(),
            path.read_text(encoding="utf-8"),
        )
        for path in paths
    )


def _split_top_level_use_items(body: str) -> list[str]:
    items = []
    depth = 0
    start = 0
    for index, character in enumerate(body):
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
        elif character == "," and depth == 0:
            items.append(body[start:index].strip())
            start = index + 1
    items.append(body[start:].strip())
    return [item for item in items if item]


def _concrete_module_identity_violations(
    source: str,
    module_name: str,
    constant_name: str,
    relative_path: str | None = None,
) -> list[str]:
    violations = []
    direct_path = re.compile(
        rf"\bcrate\s*::\s*{re.escape(module_name)}\s*::\s*{re.escape(constant_name)}\b"
    )
    if direct_path.search(source):
        violations.append(f"crate::{module_name}::{constant_name}")

    grouped_import = re.compile(
        rf"use\s+crate\s*::\s*{re.escape(module_name)}\s*::\s*\{{[^;]*\b{re.escape(constant_name)}\b[^;]*;",
        re.DOTALL,
    )
    if grouped_import.search(source):
        violations.append(f"crate::{module_name} grouped import")

    relative_facade_access = re.compile(
        rf"(?:\bsuper\s*::\s*)+{re.escape(constant_name)}\b"
    )
    concrete_subtree = f"zircon_runtime/src/{module_name}/"
    if (
        relative_path is not None
        and relative_path.startswith(concrete_subtree)
        and relative_facade_access.search(source)
    ):
        violations.append(f"relative {module_name} facade identity")

    outer_group = re.compile(
        r"use\s+crate\s*::\s*\{([^;]*)\}\s*;", re.DOTALL
    )
    aliases = set()
    for match in re.finditer(
        rf"use\s+crate\s*::\s*{re.escape(module_name)}(?:\s+as\s+([A-Za-z_]\w*))?\s*;",
        source,
    ):
        aliases.add(match.group(1) or module_name)
    for match in re.finditer(
        rf"use\s+crate\s*::\s*{re.escape(module_name)}\s*::\s*\{{([^;]*)\}}\s*;",
        source,
        re.DOTALL,
    ):
        self_import = re.search(
            r"(?:^|,)\s*self(?:\s+as\s+([A-Za-z_]\w*))?(?=\s*(?:,|$))",
            match.group(1),
        )
        if self_import:
            aliases.add(self_import.group(1) or module_name)

    crate_root_aliases = set()
    for match in outer_group.finditer(source):
        for item in _split_top_level_use_items(match.group(1)):
            normalized = re.sub(r"\s+", "", item)
            if normalized.startswith(f"{module_name}::") and re.search(
                rf"\b{re.escape(constant_name)}\b", item
            ):
                violations.append(f"crate::{{{module_name} identity use tree}}")
            root_alias = re.fullmatch(
                rf"{re.escape(module_name)}\s+as\s+([A-Za-z_]\w*)", item
            )
            if root_alias:
                aliases.add(root_alias.group(1))
            elif item == module_name:
                aliases.add(module_name)
            elif re.match(rf"{re.escape(module_name)}\s*::\s*\{{", item):
                nested_self = re.search(
                    r"\bself(?:\s+as\s+([A-Za-z_]\w*))?", item
                )
                if nested_self:
                    aliases.add(nested_self.group(1) or module_name)
            crate_root_alias = re.fullmatch(
                r"self\s+as\s+([A-Za-z_]\w*)", item
            )
            if crate_root_alias:
                crate_root_aliases.add(crate_root_alias.group(1))

    for alias in aliases:
        if re.search(
            rf"\b{re.escape(alias)}\s*::\s*{re.escape(constant_name)}\b",
            source,
        ):
            violations.append(f"crate::{module_name} alias {alias}")

    crate_root_aliases.update(
        match.group(1)
        for match in re.finditer(
            r"\buse\s+crate\s+as\s+([A-Za-z_]\w*)\s*;", source
        )
    )
    crate_root_aliases.update(
        match.group(1)
        for match in re.finditer(
            r"\bextern\s+crate\s+self\s+as\s+([A-Za-z_]\w*)\s*;", source
        )
    )
    for alias in crate_root_aliases:
        if re.search(
            rf"\b{re.escape(alias)}\s*::\s*{re.escape(module_name)}\s*::\s*{re.escape(constant_name)}\b",
            source,
        ):
            violations.append(f"crate root alias {alias}::{module_name}")
    return violations


class Frameworks05ModuleIdentityChecks:
    def test_scene_module_identity_has_one_neutral_contract_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/scene/module_identity.rs"
        )
        scene_contract = (
            REPO_ROOT / "zircon_runtime/src/core/framework/scene/mod.rs"
        ).read_text(encoding="utf-8")
        scene_module = (
            REPO_ROOT / "zircon_runtime/src/scene/module/mod.rs"
        ).read_text(encoding="utf-8")
        scene_root = (REPO_ROOT / "zircon_runtime/src/scene/mod.rs").read_text(
            encoding="utf-8"
        )
        graphics_descriptor = (
            REPO_ROOT
            / "zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs"
        ).read_text(encoding="utf-8")

        self.assertTrue(neutral_owner.is_file())
        owner_source = neutral_owner.read_text(encoding="utf-8")
        self.assertIn('pub const SCENE_MODULE_NAME: &str = "SceneModule";', owner_source)
        self.assertIn("mod module_identity;", scene_contract)
        self.assertIn("pub use module_identity::SCENE_MODULE_NAME;", scene_contract)
        self.assertNotIn("pub const SCENE_MODULE_NAME", scene_module)
        self.assertNotIn("SCENE_MODULE_NAME", scene_root)
        self.assertIn(
            "core::framework::scene::SCENE_MODULE_NAME", graphics_descriptor
        )
        self.assertNotIn("crate::scene::SCENE_MODULE_NAME", graphics_descriptor)

        scene_group_import = re.compile(
            r"use\s+crate::scene::\{[^}]*\bSCENE_MODULE_NAME\b", re.DOTALL
        )
        nested_scene_import = re.compile(
            r"use\s+crate::\{[^}]*\bscene::SCENE_MODULE_NAME\b", re.DOTALL
        )
        external_scene_group_import = re.compile(
            r"use\s+zircon_runtime::scene::\{[^}]*\bSCENE_MODULE_NAME\b", re.DOTALL
        )
        scene_module_alias_access = re.compile(
            r"(?<!:)\b(?!module_identity::)[A-Za-z_]\w*::SCENE_MODULE_NAME\b"
        )
        scene_module_name_definition = re.compile(
            r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+SCENE_MODULE_NAME\b",
            re.MULTILINE,
        )
        definition_owners = []
        for relative, source in _workspace_rust_sources():
            if scene_module_name_definition.search(source):
                definition_owners.append(relative)
            self.assertNotIn(
                "crate::scene::SCENE_MODULE_NAME",
                source,
                f"{relative} must use the neutral scene module identity owner",
            )
            self.assertNotIn(
                "zircon_runtime::scene::SCENE_MODULE_NAME",
                source,
                f"{relative} must not use the retired public scene-root identity",
            )
            self.assertIsNone(
                scene_group_import.search(source),
                f"{relative} must not import SCENE_MODULE_NAME from the scene root",
            )
            self.assertIsNone(
                nested_scene_import.search(source),
                f"{relative} must not import SCENE_MODULE_NAME through crate::scene",
            )
            self.assertIsNone(
                external_scene_group_import.search(source),
                f"{relative} must not import SCENE_MODULE_NAME from zircon_runtime::scene",
            )
            self.assertIsNone(
                scene_module_alias_access.search(source),
                f"{relative} must not access SCENE_MODULE_NAME through a scene module alias",
            )
        self.assertEqual(
            definition_owners,
            ["zircon_runtime/src/core/framework/scene/module_identity.rs"],
        )

    def test_input_module_identity_has_one_neutral_contract_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT / "zircon_runtime/src/core/framework/input/module_identity.rs"
        )
        input_contract = (
            REPO_ROOT / "zircon_runtime/src/core/framework/input/mod.rs"
        ).read_text(encoding="utf-8")
        input_descriptor = (
            REPO_ROOT / "zircon_runtime/src/input/module/descriptor.rs"
        ).read_text(encoding="utf-8")
        input_module_root = (
            REPO_ROOT / "zircon_runtime/src/input/module/mod.rs"
        ).read_text(encoding="utf-8")
        input_module_type = (
            REPO_ROOT / "zircon_runtime/src/input/module/module_type.rs"
        ).read_text(encoding="utf-8")
        input_root = (REPO_ROOT / "zircon_runtime/src/input/mod.rs").read_text(
            encoding="utf-8"
        )
        ui_module = (REPO_ROOT / "zircon_runtime/src/ui/module.rs").read_text(
            encoding="utf-8"
        )

        self.assertTrue(neutral_owner.is_file())
        owner_source = neutral_owner.read_text(encoding="utf-8")
        self.assertIn('pub const INPUT_MODULE_NAME: &str = "InputModule";', owner_source)
        self.assertIn("mod module_identity;", input_contract)
        self.assertIn("pub use module_identity::INPUT_MODULE_NAME;", input_contract)
        self.assertNotIn("pub const INPUT_MODULE_NAME", input_descriptor)
        self.assertNotIn("INPUT_MODULE_NAME", input_module_root)
        for concrete_input_owner in (input_descriptor, input_module_type):
            self.assertIn(
                "core::framework::input::INPUT_MODULE_NAME", concrete_input_owner
            )
            self.assertNotIn(
                "crate::input::INPUT_MODULE_NAME", concrete_input_owner
            )
        self.assertIn(
            "pub use crate::core::framework::input::INPUT_MODULE_NAME;", input_root
        )
        self.assertIn("core::framework::input::INPUT_MODULE_NAME", ui_module)
        self.assertNotIn("crate::input::INPUT_MODULE_NAME", ui_module)

        self.assertTrue(
            _concrete_module_identity_violations(
                "use crate::{input::{INPUT_MODULE_NAME as MODULE_NAME}};",
                "input",
                "INPUT_MODULE_NAME",
            )
        )
        for allowed in (
            "use crate::core::framework::input as input_contract;\n"
            "input_contract::INPUT_MODULE_NAME;",
            "use crate::{core::framework::input::{INPUT_MODULE_NAME}};",
            "use zircon_runtime::input as runtime_input;\n"
            "runtime_input::INPUT_MODULE_NAME;",
        ):
            self.assertEqual(
                _concrete_module_identity_violations(
                    allowed, "input", "INPUT_MODULE_NAME"
                ),
                [],
            )

        definition_pattern = re.compile(
            r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+INPUT_MODULE_NAME\b",
            re.MULTILINE,
        )
        definition_owners = []
        allowed_runtime_facades = {
            "zircon_runtime/src/input/mod.rs",
            "zircon_runtime/src/input/prelude.rs",
            "zircon_runtime/src/prelude.rs",
            "zircon_runtime/src/tests/prelude.rs",
        }
        for relative, source in _workspace_rust_sources():
            if definition_pattern.search(source):
                definition_owners.append(relative)
            if (
                relative in allowed_runtime_facades
                or relative == "zircon_runtime/src/input/tests.rs"
                or relative.startswith("zircon_runtime/src/input/tests/")
                or not relative.startswith("zircon_runtime/")
            ):
                continue
            self.assertEqual(
                _concrete_module_identity_violations(
                    source, "input", "INPUT_MODULE_NAME", relative
                ),
                [],
                f"{relative} must use the neutral input module identity owner",
            )
        self.assertEqual(
            definition_owners,
            ["zircon_runtime/src/core/framework/input/module_identity.rs"],
        )

    def test_ui_module_identity_has_one_neutral_contract_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT / "zircon_runtime/src/core/framework/ui/module_identity.rs"
        )
        ui_contract = (
            REPO_ROOT / "zircon_runtime/src/core/framework/ui.rs"
        ).read_text(encoding="utf-8")
        ui_module = (REPO_ROOT / "zircon_runtime/src/ui/module.rs").read_text(
            encoding="utf-8"
        )
        ui_root = (REPO_ROOT / "zircon_runtime/src/ui/mod.rs").read_text(
            encoding="utf-8"
        )
        ui_prelude = (REPO_ROOT / "zircon_runtime/src/ui/prelude.rs").read_text(
            encoding="utf-8"
        )
        builtin_registration = (
            REPO_ROOT
            / "zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs"
        ).read_text(encoding="utf-8")
        core_spine = (
            REPO_ROOT
            / "zircon_runtime/src/tests/runtime_absorption/builtin_modules/core_spine.rs"
        ).read_text(encoding="utf-8")

        self.assertTrue(neutral_owner.is_file())
        owner_source = neutral_owner.read_text(encoding="utf-8")
        self.assertIn('pub const UI_MODULE_NAME: &str = "UiModule";', owner_source)
        self.assertIn("mod module_identity;", ui_contract)
        self.assertIn("pub use module_identity::UI_MODULE_NAME;", ui_contract)
        self.assertNotIn("pub const UI_MODULE_NAME", ui_module)
        self.assertIn("core::framework::ui::UI_MODULE_NAME", ui_module)
        self.assertNotIn("crate::ui::UI_MODULE_NAME", ui_module)
        self.assertIn(
            "pub use crate::core::framework::ui::UI_MODULE_NAME;", ui_root
        )
        self.assertNotRegex(
            ui_root,
            r"pub use module::\{[^}]*\bUI_MODULE_NAME\b",
        )
        self.assertIn("pub use super::UI_MODULE_NAME;", ui_prelude)
        self.assertNotRegex(
            ui_prelude,
            r"pub use super::module::\{[^}]*\bUI_MODULE_NAME\b",
        )
        for runtime_internal in (builtin_registration, core_spine):
            self.assertIn("core::framework::ui::UI_MODULE_NAME", runtime_internal)
            self.assertNotIn("crate::ui::UI_MODULE_NAME", runtime_internal)

        self.assertTrue(
            _concrete_module_identity_violations(
                "use crate::{ui::{UI_MODULE_NAME as MODULE_NAME}};",
                "ui",
                "UI_MODULE_NAME",
            )
        )
        for denied in (
            "const MODULE_NAME: &str = super::UI_MODULE_NAME;",
            "const MODULE_NAME: &str = super::super::UI_MODULE_NAME;",
            "const MODULE_NAME: &str = crate :: ui :: UI_MODULE_NAME;",
            "use crate :: ui as runtime_ui;\n"
            "const MODULE_NAME: &str = runtime_ui :: UI_MODULE_NAME;",
            "use crate :: { ui :: UI_MODULE_NAME };",
            "use crate as runtime;\n"
            "const MODULE_NAME: &str = runtime::ui::UI_MODULE_NAME;",
            "extern crate self as runtime;\n"
            "const MODULE_NAME: &str = runtime::ui::UI_MODULE_NAME;",
            "use crate::{self as runtime};\n"
            "const MODULE_NAME: &str = runtime::ui::UI_MODULE_NAME;",
        ):
            self.assertTrue(
                _concrete_module_identity_violations(
                    denied,
                    "ui",
                    "UI_MODULE_NAME",
                    "zircon_runtime/src/ui/concrete_child.rs",
                )
            )
        for allowed in (
            "use crate::core::framework::ui as ui_contract;\n"
            "ui_contract::UI_MODULE_NAME;",
            "use crate::{core::framework::ui::{UI_MODULE_NAME}};",
            "use zircon_runtime::ui as runtime_ui;\n"
            "runtime_ui::UI_MODULE_NAME;",
            "const MODULE_NAME: &str = super::UI_MODULE_NAME;",
        ):
            self.assertEqual(
                _concrete_module_identity_violations(
                    allowed,
                    "ui",
                    "UI_MODULE_NAME",
                    "zircon_runtime/src/core/framework/ui/neutral_child.rs",
                ),
                [],
            )

        definition_pattern = re.compile(
            r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+UI_MODULE_NAME\b",
            re.MULTILINE,
        )
        definition_owners = []
        allowed_runtime_facades = {
            "zircon_runtime/src/ui/mod.rs",
            "zircon_runtime/src/ui/prelude.rs",
            "zircon_runtime/src/prelude.rs",
            "zircon_runtime/src/tests/prelude.rs",
            "zircon_runtime/src/tests/ui_boundary/runtime_host.rs",
        }
        for relative, source in _workspace_rust_sources():
            if definition_pattern.search(source):
                definition_owners.append(relative)
            if (
                relative in allowed_runtime_facades
                or relative == "zircon_runtime/src/ui/tests.rs"
                or relative.startswith("zircon_runtime/src/ui/tests/")
                or not relative.startswith("zircon_runtime/")
            ):
                continue
            self.assertEqual(
                _concrete_module_identity_violations(
                    source, "ui", "UI_MODULE_NAME", relative
                ),
                [],
                f"{relative} must use the neutral ui module identity owner",
            )
        self.assertEqual(
            definition_owners,
            ["zircon_runtime/src/core/framework/ui/module_identity.rs"],
        )

    def test_foundation_module_identity_has_one_neutral_contract_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/foundation/module_identity.rs"
        )
        foundation_contract_path = (
            REPO_ROOT / "zircon_runtime/src/core/framework/foundation/mod.rs"
        )
        legacy_contract = (
            REPO_ROOT / "zircon_runtime/src/core/framework/foundation.rs"
        )
        foundation_module = (
            REPO_ROOT / "zircon_runtime/src/foundation/module.rs"
        ).read_text(encoding="utf-8")
        foundation_root = (
            REPO_ROOT / "zircon_runtime/src/foundation/mod.rs"
        ).read_text(encoding="utf-8")
        asset_module = (
            REPO_ROOT / "zircon_runtime/src/asset/module.rs"
        ).read_text(encoding="utf-8")
        platform_module = (
            REPO_ROOT / "zircon_runtime/src/platform/module.rs"
        ).read_text(encoding="utf-8")
        builtin_registration = (
            REPO_ROOT
            / "zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs"
        ).read_text(encoding="utf-8")
        core_spine = (
            REPO_ROOT
            / "zircon_runtime/src/tests/runtime_absorption/builtin_modules/core_spine.rs"
        ).read_text(encoding="utf-8")

        self.assertFalse(legacy_contract.exists())
        self.assertTrue(foundation_contract_path.is_file())
        self.assertTrue(neutral_owner.is_file())
        foundation_contract = foundation_contract_path.read_text(encoding="utf-8")
        owner_source = neutral_owner.read_text(encoding="utf-8")
        self.assertIn(
            'pub const FOUNDATION_MODULE_NAME: &str = "FoundationModule";',
            owner_source,
        )
        for child in (
            "config_manager",
            "config_persistence_report",
            "event_manager",
            "module_identity",
        ):
            self.assertIn(f"mod {child};", foundation_contract)
        self.assertIn("pub use config_manager::ConfigManager;", foundation_contract)
        self.assertIn(
            "pub use config_persistence_report::ConfigPersistenceReport;",
            foundation_contract,
        )
        self.assertIn("pub use event_manager::EventManager;", foundation_contract)
        self.assertIn(
            "pub use module_identity::FOUNDATION_MODULE_NAME;", foundation_contract
        )
        self.assertNotIn("pub const FOUNDATION_MODULE_NAME", foundation_module)
        self.assertRegex(
            foundation_module,
            r"core::framework::foundation::\{[^}]*\bFOUNDATION_MODULE_NAME\b",
        )
        self.assertNotIn("crate::foundation::FOUNDATION_MODULE_NAME", foundation_module)
        self.assertIn(
            "pub use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;",
            foundation_root,
        )
        self.assertNotRegex(
            foundation_root,
            r"pub use module::\{[^}]*\bFOUNDATION_MODULE_NAME\b",
        )
        for runtime_internal in (
            asset_module,
            platform_module,
            builtin_registration,
            core_spine,
        ):
            self.assertIn(
                "core::framework::foundation::FOUNDATION_MODULE_NAME",
                runtime_internal,
            )
            self.assertNotIn("crate::foundation::FOUNDATION_MODULE_NAME", runtime_internal)

        for denied in (
            "const MODULE_NAME: &str = crate :: foundation :: FOUNDATION_MODULE_NAME;",
            "use crate :: foundation as runtime_foundation;\n"
            "const MODULE_NAME: &str = runtime_foundation :: FOUNDATION_MODULE_NAME;",
            "use crate :: { foundation :: FOUNDATION_MODULE_NAME };",
            "use crate::{self as runtime};\n"
            "const MODULE_NAME: &str = runtime::foundation::FOUNDATION_MODULE_NAME;",
            "const MODULE_NAME: &str = super::super::FOUNDATION_MODULE_NAME;",
        ):
            self.assertTrue(
                _concrete_module_identity_violations(
                    denied,
                    "foundation",
                    "FOUNDATION_MODULE_NAME",
                    "zircon_runtime/src/foundation/concrete_child.rs",
                )
            )
        for allowed in (
            "use crate::core::framework::foundation as foundation_contract;\n"
            "foundation_contract::FOUNDATION_MODULE_NAME;",
            "use crate::{core::framework::foundation::{FOUNDATION_MODULE_NAME}};",
            "use zircon_runtime::foundation as runtime_foundation;\n"
            "runtime_foundation::FOUNDATION_MODULE_NAME;",
            "const MODULE_NAME: &str = super::FOUNDATION_MODULE_NAME;",
        ):
            self.assertEqual(
                _concrete_module_identity_violations(
                    allowed,
                    "foundation",
                    "FOUNDATION_MODULE_NAME",
                    "zircon_runtime/src/core/framework/foundation/neutral_child.rs",
                ),
                [],
            )

        definition_pattern = re.compile(
            r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+FOUNDATION_MODULE_NAME\b",
            re.MULTILINE,
        )
        definition_owners = []
        allowed_runtime_facades = {
            "zircon_runtime/src/foundation/mod.rs",
            "zircon_runtime/src/prelude.rs",
            "zircon_runtime/src/tests/prelude.rs",
        }
        for relative, source in _workspace_rust_sources():
            if definition_pattern.search(source):
                definition_owners.append(relative)
            if (
                relative in allowed_runtime_facades
                or relative == "zircon_runtime/src/foundation/tests.rs"
                or relative.startswith("zircon_runtime/src/foundation/tests/")
                or relative.startswith("zircon_runtime/tests/")
                or not relative.startswith("zircon_runtime/")
            ):
                continue
            self.assertEqual(
                _concrete_module_identity_violations(
                    source,
                    "foundation",
                    "FOUNDATION_MODULE_NAME",
                    relative,
                ),
                [],
                f"{relative} must use the neutral foundation module identity owner",
            )
        self.assertEqual(
            definition_owners,
            ["zircon_runtime/src/core/framework/foundation/module_identity.rs"],
        )
