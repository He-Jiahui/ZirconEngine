import re
import subprocess
import unittest
from dataclasses import dataclass
from pathlib import Path

from tools.runtime_domain_dependency_audit import _rust_code_view, _rust_use_paths


REPO_ROOT = Path(__file__).resolve().parents[2]
CONTRACT_ROOT = REPO_ROOT / "zircon_runtime/src/core/framework/camera_controller"
INPUT_OWNER_ROOT = REPO_ROOT / "zircon_runtime/src/input/camera_controller"
MOVED_CONTROLLER_NAMES = {
    "FreeCameraController",
    "OrbitCameraController",
    "PanCameraController",
}
OLD_OWNER_PREFIXES = tuple(
    prefix + (name,)
    for prefix in (
        ("crate", "core", "framework", "camera_controller"),
        ("zircon_runtime", "core", "framework", "camera_controller"),
    )
    for name in MOVED_CONTROLLER_NAMES
)
OLD_OWNER_ROOTS = {prefix[:-1] for prefix in OLD_OWNER_PREFIXES}
NEW_OWNER_ROOTS = (
    ("crate", "input", "camera_controller"),
    ("zircon_runtime", "input", "camera_controller"),
)
UNQUALIFIED_CONTROLLER = re.compile(
    rf"(?<![:\w])(?:{'|'.join(sorted(MOVED_CONTROLLER_NAMES))})\b"
)
CONTROLLER_DECLARATION = re.compile(
    rf"\bstruct\s+(?P<name>{'|'.join(sorted(MOVED_CONTROLLER_NAMES))})\b"
)
INLINE_MODULE = re.compile(r"\bmod\s+(?P<name>[A-Za-z_]\w*)\s*\{")
EXTERN_CRATE = re.compile(
    r"\bextern\s+crate\s+(?P<crate>self|[A-Za-z_]\w*)"
    r"(?:\s+as\s+(?P<alias>[A-Za-z_]\w*))?\s*;"
)
QUALIFIED_PATH = re.compile(
    r"\b[A-Za-z_]\w*\b(?:\s*::\s*(?:[A-Za-z_]\w*|\*))+"
)


@dataclass
class RustScope:
    module_path: tuple[str, ...]
    start: int
    end: int
    parent: int | None
    is_module: bool


@dataclass(frozen=True)
class RustAlias:
    path: tuple[str, ...]
    scope: int


@dataclass(frozen=True)
class RustUsePath:
    path: tuple[str, ...]
    alias: str | None
    line: int
    scope: int


def _rust_scopes(
    code_view: str,
    file_module_path: tuple[str, ...],
) -> list[RustScope]:
    module_openings = {}
    for declaration in INLINE_MODULE.finditer(code_view):
        opening_brace = code_view.find("{", declaration.start(), declaration.end())
        module_openings[opening_brace] = declaration.group("name")

    scopes = [RustScope(file_module_path, 0, len(code_view), None, True)]
    stack = [0]
    for position, character in enumerate(code_view):
        if character == "{":
            parent = stack[-1]
            module_name = module_openings.get(position)
            module_path = scopes[parent].module_path
            if module_name is not None:
                module_path += (module_name,)
            scopes.append(
                RustScope(
                    module_path,
                    position + 1,
                    len(code_view),
                    parent,
                    module_name is not None,
                )
            )
            stack.append(len(scopes) - 1)
        elif character == "}" and len(stack) > 1:
            scopes[stack.pop()].end = position
    return scopes


def _scope_at(scopes: list[RustScope], position: int) -> int:
    return max(
        (
            index
            for index, scope in enumerate(scopes)
            if scope.start <= position < scope.end
        ),
        key=lambda index: scopes[index].start,
    )


def _nearest_module_scope(scopes: list[RustScope], scope: int) -> int:
    current = scope
    while not scopes[current].is_module:
        parent = scopes[current].parent
        if parent is None:
            return current
        current = parent
    return current


def _parent_module_scope(scopes: list[RustScope], scope: int) -> int | None:
    parent = scopes[scope].parent
    while parent is not None and not scopes[parent].is_module:
        parent = scopes[parent].parent
    return parent


def _module_target(
    scopes: list[RustScope],
    scope: int,
    super_count: int,
) -> tuple[tuple[str, ...], int | None]:
    module_scope: int | None = _nearest_module_scope(scopes, scope)
    module_path = scopes[module_scope].module_path
    for _ in range(super_count):
        parent = (
            _parent_module_scope(scopes, module_scope)
            if module_scope is not None
            else None
        )
        if parent is not None:
            module_scope = parent
            module_path = scopes[parent].module_path
        else:
            module_scope = None
            module_path = module_path[:-1]
    return module_path, module_scope


def _line_start_offsets(code_view: str) -> list[int]:
    offsets = [0]
    offsets.extend(match.end() for match in re.finditer("\n", code_view))
    return offsets


def _effective_use_alias(path: tuple[str, ...], alias: str | None) -> str | None:
    if alias is not None:
        return alias
    if not path or path[-1] == "*":
        return None
    if path[-1] == "self" and len(path) > 1:
        return path[-2]
    return path[-1]


def _source_aliases(
    code_view: str,
    scopes: list[RustScope],
) -> tuple[dict[int, dict[str, RustAlias]], list[RustUsePath]]:
    aliases = {index: {} for index in range(len(scopes))}
    use_paths = []
    line_offsets = _line_start_offsets(code_view)
    for path, alias, line in _rust_use_paths(code_view):
        scope = _scope_at(scopes, line_offsets[line - 1])
        use_paths.append(RustUsePath(path, alias, line, scope))
        effective_alias = _effective_use_alias(path, alias)
        if effective_alias is not None:
            aliases[scope][effective_alias] = RustAlias(path, scope)

    for declaration in EXTERN_CRATE.finditer(code_view):
        scope = _scope_at(scopes, declaration.start())
        crate_name = declaration.group("crate")
        alias = declaration.group("alias") or crate_name
        target = ("crate",) if crate_name == "self" else (crate_name,)
        aliases[scope][alias] = RustAlias(target, scope)
    return aliases, use_paths


def _visible_scope_indices(scopes: list[RustScope], scope: int):
    module_scope = _nearest_module_scope(scopes, scope)
    current: int | None = scope
    while current is not None:
        yield current
        if current == module_scope:
            return
        current = scopes[current].parent


def _visible_alias(
    aliases: dict[int, dict[str, RustAlias]],
    scopes: list[RustScope],
    scope: int,
    name: str,
) -> RustAlias | None:
    for visible_scope in _visible_scope_indices(scopes, scope):
        if name in aliases[visible_scope]:
            return aliases[visible_scope][name]
    return None


def _resolve_module_member(
    path: tuple[str, ...],
    module_path: tuple[str, ...],
    module_scope: int | None,
    aliases: dict[int, dict[str, RustAlias]],
    scopes: list[RustScope],
    glob_members: dict[int, tuple[tuple[str, ...], ...]],
    seen: frozenset[tuple[int, str]],
) -> tuple[str, ...]:
    if path and module_scope is not None and path[0] in aliases[module_scope]:
        alias = aliases[module_scope][path[0]]
        key = (alias.scope, path[0])
        if key not in seen:
            target = _resolve_path(
                alias.path,
                alias.scope,
                aliases,
                scopes,
                glob_members,
                seen | {key},
            )
            return _resolve_path(
                target + path[1:],
                alias.scope,
                aliases,
                scopes,
                glob_members,
                seen | {key},
            )
    if path and module_scope is not None and path[0] in MOVED_CONTROLLER_NAMES:
        for owner_root in glob_members.get(module_scope, ()):
            return (*owner_root, *path)
    return ("crate", *module_path, *path)


def _resolve_path(
    path: tuple[str, ...],
    scope: int,
    aliases: dict[int, dict[str, RustAlias]],
    scopes: list[RustScope],
    glob_members: dict[int, tuple[tuple[str, ...], ...]] | None = None,
    seen: frozenset[tuple[int, str]] = frozenset(),
) -> tuple[str, ...]:
    glob_members = glob_members or {}
    if not path:
        return path
    if path[0] == "crate":
        return path
    if path[0] == "self":
        module_path, module_scope = _module_target(scopes, scope, 0)
        return _resolve_module_member(
            path[1:],
            module_path,
            module_scope,
            aliases,
            scopes,
            glob_members,
            seen,
        )
    if path[0] == "super":
        index = 0
        while index < len(path) and path[index] == "super":
            index += 1
        if index < len(path) and path[index] == "self":
            index += 1
        module_path, module_scope = _module_target(scopes, scope, index)
        return _resolve_module_member(
            path[index:],
            module_path,
            module_scope,
            aliases,
            scopes,
            glob_members,
            seen,
        )

    alias = _visible_alias(aliases, scopes, scope, path[0])
    if alias is None:
        return path
    key = (alias.scope, path[0])
    if key in seen:
        return path
    target = _resolve_path(
        alias.path,
        alias.scope,
        aliases,
        scopes,
        glob_members,
        seen | {key},
    )
    return _resolve_path(
        target + path[1:],
        alias.scope,
        aliases,
        scopes,
        glob_members,
        seen | {key},
    )


def _qualified_path_segments(path: str) -> tuple[str, ...]:
    return tuple(re.findall(r"[A-Za-z_]\w*|\*", path))


def _qualified_owner_lines(
    code_view: str,
    scopes: list[RustScope],
    aliases: dict[int, dict[str, RustAlias]],
    owner_paths: tuple[tuple[str, ...], ...],
    glob_members: dict[int, tuple[tuple[str, ...], ...]],
) -> set[int]:
    lines = set()
    for reference in QUALIFIED_PATH.finditer(code_view):
        scope = _scope_at(scopes, reference.start())
        path = _resolve_path(
            _qualified_path_segments(reference.group()),
            scope,
            aliases,
            scopes,
            glob_members,
        )
        if any(path[: len(owner)] == owner for owner in owner_paths):
            lines.add(code_view.count("\n", 0, reference.start()) + 1)
    return lines


def _glob_is_visible(
    glob_scopes: set[int],
    scopes: list[RustScope],
    scope: int,
) -> bool:
    return any(
        visible_scope in glob_scopes
        for visible_scope in _visible_scope_indices(scopes, scope)
    )


def _owner_glob_members(
    use_paths: list[RustUsePath],
    aliases: dict[int, dict[str, RustAlias]],
    scopes: list[RustScope],
    owner_roots: set[tuple[str, ...]] | tuple[tuple[str, ...], ...],
) -> dict[int, tuple[tuple[str, ...], ...]]:
    members: dict[int, set[tuple[str, ...]]] = {}
    for use_path in use_paths:
        resolved = _resolve_path(
            use_path.path,
            use_path.scope,
            aliases,
            scopes,
        )
        if resolved and resolved[-1] == "*" and resolved[:-1] in owner_roots:
            members.setdefault(use_path.scope, set()).add(resolved[:-1])
    return {
        scope: tuple(sorted(roots))
        for scope, roots in members.items()
    }


def old_controller_import_lines(
    source: str,
    file_module_path: tuple[str, ...] = (),
) -> list[int]:
    code_view = _rust_code_view(source)
    scopes = _rust_scopes(code_view, file_module_path)
    aliases, use_paths = _source_aliases(code_view, scopes)
    glob_members = _owner_glob_members(
        use_paths,
        aliases,
        scopes,
        OLD_OWNER_ROOTS,
    )
    lines = _qualified_owner_lines(
        code_view,
        scopes,
        aliases,
        OLD_OWNER_PREFIXES,
        glob_members,
    )
    for use_path in use_paths:
        resolved = _resolve_path(
            use_path.path,
            use_path.scope,
            aliases,
            scopes,
            glob_members,
        )
        if resolved in OLD_OWNER_PREFIXES:
            lines.add(use_path.line)

    for reference in UNQUALIFIED_CONTROLLER.finditer(code_view):
        scope = _scope_at(scopes, reference.start())
        if _glob_is_visible(set(glob_members), scopes, scope):
            lines.add(code_view.count("\n", 0, reference.start()) + 1)
    return sorted(lines)


def new_owner_forwarding_lines(
    source: str,
    file_module_path: tuple[str, ...] = (),
) -> list[int]:
    code_view = _rust_code_view(source)
    scopes = _rust_scopes(code_view, file_module_path)
    aliases, use_paths = _source_aliases(code_view, scopes)
    glob_members = _owner_glob_members(
        use_paths,
        aliases,
        scopes,
        NEW_OWNER_ROOTS,
    )
    lines = _qualified_owner_lines(
        code_view,
        scopes,
        aliases,
        NEW_OWNER_ROOTS,
        glob_members,
    )
    for use_path in use_paths:
        resolved = _resolve_path(
            use_path.path,
            use_path.scope,
            aliases,
            scopes,
            glob_members,
        )
        if any(resolved[: len(root)] == root for root in NEW_OWNER_ROOTS):
            lines.add(use_path.line)
    return sorted(lines)


def rust_module_path(path: Path) -> tuple[str, ...]:
    relative_parts = path.relative_to(REPO_ROOT).parts
    source_index = max(
        index for index, segment in enumerate(relative_parts) if segment == "src"
    )
    source_parts = relative_parts[source_index + 1 :]
    filename = source_parts[-1]
    module_path = list(source_parts[:-1])
    if filename not in {"lib.rs", "main.rs", "mod.rs"}:
        module_path.append(Path(filename).stem)
    return tuple(module_path)


def product_rust_candidate_paths() -> list[Path]:
    result = subprocess.run(
        [
            "git",
            "grep",
            "-l",
            "--untracked",
            "-e",
            "FreeCameraController",
            "-e",
            "PanCameraController",
            "-e",
            "OrbitCameraController",
            "--",
            ":(glob)zircon_runtime/**/*.rs",
            ":(glob)zircon_app/**/*.rs",
            ":(glob)zircon_editor/**/*.rs",
            ":(glob)zircon_plugins/**/*.rs",
        ],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode not in {0, 1}:
        raise RuntimeError(result.stderr.strip() or "git grep failed")
    return [REPO_ROOT / path for path in result.stdout.splitlines()]


def controller_declaration_lines(source: str) -> dict[str, list[int]]:
    code_view = _rust_code_view(source)
    declarations = {name: [] for name in MOVED_CONTROLLER_NAMES}
    for declaration in CONTROLLER_DECLARATION.finditer(code_view):
        declarations[declaration.group("name")].append(
            code_view.count("\n", 0, declaration.start()) + 1
        )
    return declarations


def controller_declaration_owners() -> dict[str, list[str]]:
    owners = {name: [] for name in MOVED_CONTROLLER_NAMES}
    for path in product_rust_candidate_paths():
        declarations = controller_declaration_lines(path.read_text(encoding="utf-8"))
        relative_path = path.relative_to(REPO_ROOT).as_posix()
        for name, lines in declarations.items():
            owners[name].extend(f"{relative_path}:{line}" for line in lines)
    return owners


class Frameworks01CameraControllerOwnerBoundaryTests(unittest.TestCase):
    def test_controller_behavior_has_one_runtime_input_owner(self) -> None:
        input_mod = (REPO_ROOT / "zircon_runtime/src/input/mod.rs").read_text(
            encoding="utf-8"
        )
        owner_mod = (INPUT_OWNER_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertRegex(input_mod, r"(?m)^pub mod camera_controller;$")
        self.assertIn("pub use free::FreeCameraController;", owner_mod)
        self.assertIn("pub use orbit::OrbitCameraController;", owner_mod)
        self.assertIn("pub use pan::PanCameraController;", owner_mod)
        for controller_kind in ("free", "orbit", "pan"):
            self.assertTrue((INPUT_OWNER_ROOT / controller_kind / "controller.rs").is_file())
            self.assertFalse((CONTRACT_ROOT / controller_kind / "controller.rs").exists())

        expected_owner_paths = {
            "FreeCameraController": "zircon_runtime/src/input/camera_controller/free/controller.rs",
            "OrbitCameraController": "zircon_runtime/src/input/camera_controller/orbit/controller.rs",
            "PanCameraController": "zircon_runtime/src/input/camera_controller/pan/controller.rs",
        }
        declaration_owners = controller_declaration_owners()
        for name, expected_path in expected_owner_paths.items():
            self.assertEqual(1, len(declaration_owners[name]), declaration_owners[name])
            self.assertEqual(
                expected_path,
                declaration_owners[name][0].rsplit(":", 1)[0],
            )

    def test_contract_modules_export_only_dtos(self) -> None:
        root_mod = (CONTRACT_ROOT / "mod.rs").read_text(encoding="utf-8")
        free_mod = (CONTRACT_ROOT / "free/mod.rs").read_text(encoding="utf-8")
        orbit_mod = (CONTRACT_ROOT / "orbit/mod.rs").read_text(encoding="utf-8")
        pan_mod = (CONTRACT_ROOT / "pan/mod.rs").read_text(encoding="utf-8")

        self.assertNotIn("FreeCameraController", root_mod)
        self.assertNotIn("OrbitCameraController", root_mod)
        self.assertNotIn("PanCameraController", root_mod)
        self.assertNotIn("mod controller;", free_mod)
        self.assertNotIn("mod controller;", orbit_mod)
        self.assertNotIn("mod controller;", pan_mod)
        self.assertIn("FreeCameraInput", root_mod)
        self.assertIn("OrbitCameraInput", root_mod)
        self.assertIn("PanCameraInput", root_mod)

    def test_framework_contract_tree_does_not_forward_input_implementation(self) -> None:
        violations = []
        for path in CONTRACT_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            source_lines = source.splitlines()
            for line_number in new_owner_forwarding_lines(
                source,
                rust_module_path(path),
            ):
                violations.append(
                    f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: "
                    f"{source_lines[line_number - 1].strip()}"
                )
        self.assertEqual(
            [],
            violations,
            "framework camera contracts forward input implementation:\n"
            + "\n".join(violations),
        )

    def test_product_sources_do_not_import_moved_controllers_from_contracts(self) -> None:
        violations = []
        for path in product_rust_candidate_paths():
            source = path.read_text(encoding="utf-8")
            source_lines = source.splitlines()
            for line_number in old_controller_import_lines(
                source,
                rust_module_path(path),
            ):
                violations.append(
                    f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: "
                    f"{source_lines[line_number - 1].strip()}"
                )
        self.assertEqual([], violations, "old camera controller owner remains:\n" + "\n".join(violations))

    def test_old_owner_scanner_handles_grouped_glob_alias_and_qualified_paths(self) -> None:
        grouped = """
use zircon_runtime::core::framework::camera_controller::{
    FreeCameraController, FreeCameraInput, OrbitCameraController, PanCameraController,
};
"""
        globbed = """
use crate::core::framework::camera_controller::*;
type Controller = PanCameraController;
"""
        qualified = """
type Controller = zircon_runtime::core::framework::camera_controller::FreeCameraController;
"""
        aliased = """
use zircon_runtime::core::framework::camera_controller as camera;
type Controller = camera::PanCameraController;
"""
        grouped_alias = """
use zircon_runtime::core::framework::{camera_controller as camera, time::Time};
type Controller = camera::FreeCameraController;
"""
        chained_alias = """
use zircon_runtime as runtime;
use runtime::core as engine_core;
use engine_core::framework as contracts;
type Controller = contracts::camera_controller::PanCameraController;
"""
        implicit_leaf_alias = """
use crate::core::framework;
use framework::camera_controller;
type Controller = camera_controller::FreeCameraController;
"""

        self.assertEqual([3], old_controller_import_lines(grouped))
        self.assertEqual([3], old_controller_import_lines(globbed))
        self.assertEqual([2], old_controller_import_lines(qualified))
        self.assertEqual([3], old_controller_import_lines(aliased))
        self.assertEqual([3], old_controller_import_lines(grouped_alias))
        self.assertEqual([5], old_controller_import_lines(chained_alias))
        self.assertEqual([4], old_controller_import_lines(implicit_leaf_alias))

    def test_new_owner_forwarding_scanner_handles_alias_and_glob_paths(self) -> None:
        direct_glob = """
pub use crate::input::camera_controller::*;
"""
        root_alias = """
use crate::input as runtime_input;
pub use runtime_input::camera_controller::{FreeCameraController, PanCameraController};
"""
        module_alias = """
use zircon_runtime::input::camera_controller as controllers;
pub type CompatController = controllers::FreeCameraController;
"""

        self.assertEqual([2], new_owner_forwarding_lines(direct_glob))
        self.assertEqual([3], new_owner_forwarding_lines(root_alias))
        self.assertEqual([2, 3], new_owner_forwarding_lines(module_alias))

    def test_declaration_scanner_counts_duplicates_and_ignores_non_code(self) -> None:
        source = r'''
// pub struct FreeCameraController;
const DOC: &str = "struct PanCameraController";
pub struct FreeCameraController;
struct FreeCameraController;
pub(crate) struct PanCameraController;
pub struct OrbitCameraController;
'''

        self.assertEqual(
            {
                "FreeCameraController": [4, 5],
                "OrbitCameraController": [7],
                "PanCameraController": [6],
            },
            controller_declaration_lines(source),
        )

    def test_old_owner_scanner_handles_relative_module_and_chained_paths(self) -> None:
        direct = """
type Controller = super::core::framework::camera_controller::FreeCameraController;
"""
        module_alias = """
use super::core::framework::camera_controller as camera;
type Controller = camera::PanCameraController;
"""
        chained_alias = """
use super::core as engine_core;
use engine_core::framework as contracts;
type Controller = contracts::camera_controller::FreeCameraController;
"""
        self_relative = """
use self::camera_controller as camera;
type Controller = camera::PanCameraController;
"""

        self.assertEqual([2], old_controller_import_lines(direct, ("tests",)))
        self.assertEqual([3], old_controller_import_lines(module_alias, ("tests",)))
        self.assertEqual([4], old_controller_import_lines(chained_alias, ("tests",)))
        self.assertEqual(
            [3],
            old_controller_import_lines(
                self_relative,
                ("core", "framework"),
            ),
        )

    def test_new_owner_forwarding_scanner_handles_relative_glob(self) -> None:
        relative_glob = """
pub use super::super::super::input::camera_controller::*;
"""

        self.assertEqual(
            [2],
            new_owner_forwarding_lines(
                relative_glob,
                ("core", "framework", "camera_controller"),
            ),
        )

    def test_old_owner_aliases_are_isolated_between_inline_modules(self) -> None:
        source = """
mod dto {
    use crate::core::framework::camera_controller as camera;
    type Input = camera::FreeCameraInput;
}
mod implementation {
    use crate::input::camera_controller as camera;
    type Controller = camera::FreeCameraController;
}
"""

        self.assertEqual([], old_controller_import_lines(source))

    def test_old_owner_scanner_resolves_parent_module_aliases(self) -> None:
        explicit_alias = """
use crate::core::framework::camera_controller as camera;
mod child {
    type Controller = super::camera::FreeCameraController;
}
"""
        implicit_alias = """
use crate::core::framework;
mod child {
    type Controller = super::framework::camera_controller::PanCameraController;
}
"""
        chained_alias = """
use crate::core as engine_core;
use engine_core::framework as contracts;
mod child {
    type Controller = super::contracts::camera_controller::FreeCameraController;
}
"""

        self.assertEqual([4], old_controller_import_lines(explicit_alias))
        self.assertEqual([4], old_controller_import_lines(implicit_alias))
        self.assertEqual([5], old_controller_import_lines(chained_alias))

    def test_new_owner_forwarding_scanner_resolves_parent_module_alias(self) -> None:
        source = """
use crate::input as runtime_input;
mod child {
    pub use super::runtime_input::camera_controller::*;
}
"""

        self.assertEqual(
            [4],
            new_owner_forwarding_lines(
                source,
                ("core", "framework", "camera_controller"),
            ),
        )

    def test_owner_scanners_resolve_extern_crate_aliases(self) -> None:
        old_owner = """
extern crate zircon_runtime as runtime;
type Controller = runtime::core::framework::camera_controller::PanCameraController;
"""
        new_owner = """
extern crate self as runtime;
pub use runtime::input::camera_controller::*;
"""

        self.assertEqual([3], old_controller_import_lines(old_owner))
        self.assertEqual([3], new_owner_forwarding_lines(new_owner))

    def test_old_owner_aliases_are_isolated_between_function_scopes(self) -> None:
        source = """
fn dto_contract() {
    use crate::core::framework::camera_controller as camera;
    type Input = camera::FreeCameraInput;
}
fn implementation() {
    use crate::input::camera_controller as camera;
    type Controller = camera::FreeCameraController;
}
"""

        self.assertEqual([], old_controller_import_lines(source))

    def test_old_owner_scanner_resolves_parent_glob_members(self) -> None:
        qualified_reference = """
use crate::core::framework::camera_controller::*;
mod child {
    type Controller = super::FreeCameraController;
}
"""
        child_import = """
use crate::core::framework::camera_controller::*;
mod child {
    use super::PanCameraController;
}
"""

        self.assertEqual([4], old_controller_import_lines(qualified_reference))
        self.assertEqual([4], old_controller_import_lines(child_import))


if __name__ == "__main__":
    unittest.main()
