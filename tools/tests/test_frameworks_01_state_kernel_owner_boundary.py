import re
import subprocess
import unittest
from pathlib import Path

from tools.runtime_domain_dependency_audit import _rust_code_view, _rust_use_paths


REPO_ROOT = Path(__file__).resolve().parents[2]
OLD_STATE_ROOT = REPO_ROOT / "zircon_runtime/src/core/framework/state"
KERNEL_STATE_ROOT = REPO_ROOT / "zircon_runtime/src/core/runtime/state_machine"
PRODUCT_SOURCE_ROOTS = (
    "zircon_runtime/",
    "zircon_app/",
    "zircon_editor/",
    "zircon_plugins/",
)
REQUIRED_STATE_FILES = {
    "hook.rs",
    "hook_index.rs",
    "machine.rs",
    "mod.rs",
    "next_state.rs",
    "on_enter.rs",
    "on_exit.rs",
    "on_transition.rs",
    "registry.rs",
    "state.rs",
    "state_spec.rs",
    "state_transition_event.rs",
}
OLD_USE_PREFIXES = (
    ("crate", "core", "framework", "state"),
    ("zircon_runtime", "core", "framework", "state"),
)
OLD_QUALIFIED_PATH = re.compile(
    r"\b(?:crate|zircon_runtime)\s*::\s*core\s*::\s*framework\s*::\s*state\b"
)


def old_state_owner_lines(source: str) -> list[int]:
    code_view = _rust_code_view(source)
    use_paths = _rust_use_paths(code_view)
    aliases: dict[str, tuple[str, ...]] = {}

    def resolve_alias(path: tuple[str, ...]) -> tuple[str, ...]:
        resolved = path
        seen: set[str] = set()
        while resolved and resolved[0] in aliases and resolved[0] not in seen:
            seen.add(resolved[0])
            resolved = aliases[resolved[0]] + resolved[1:]
        return resolved

    for _ in range(len(use_paths) + 1):
        changed = False
        for path, alias, _line in use_paths:
            if alias is None:
                continue
            resolved = resolve_alias(path)
            if not any(
                prefix[: len(resolved)] == resolved for prefix in OLD_USE_PREFIXES
            ):
                continue
            if aliases.get(alias) != resolved:
                aliases[alias] = resolved
                changed = True
        if not changed:
            break

    lines = set()
    for path, _alias, line in use_paths:
        resolved = resolve_alias(path)
        if any(resolved[: len(prefix)] == prefix for prefix in OLD_USE_PREFIXES):
            lines.add(line)

    for reference in OLD_QUALIFIED_PATH.finditer(code_view):
        lines.add(code_view.count("\n", 0, reference.start()) + 1)
    for alias, target in aliases.items():
        for prefix in OLD_USE_PREFIXES:
            if prefix[: len(target)] != target:
                continue
            suffix = prefix[len(target) :]
            if not suffix:
                continue
            alias_path = re.compile(
                rf"\b{re.escape(alias)}\s*"
                + "".join(rf"::\s*{re.escape(segment)}\s*" for segment in suffix)
            )
            for reference in alias_path.finditer(code_view):
                lines.add(code_view.count("\n", 0, reference.start()) + 1)
    return sorted(lines)


def product_rust_candidate_paths() -> list[Path]:
    result = subprocess.run(
        [
            "git",
            "grep",
            "-l",
            "--untracked",
            "--all-match",
            "-w",
            "-e",
            "framework",
            "-e",
            "state",
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
    return [
        REPO_ROOT / relative_path
        for relative_path in result.stdout.splitlines()
        if relative_path.startswith(PRODUCT_SOURCE_ROOTS)
    ]


class Frameworks01StateKernelOwnerBoundaryTests(unittest.TestCase):
    def test_state_machine_has_one_kernel_owner_and_no_old_framework_owner(self) -> None:
        self.assertFalse(
            OLD_STATE_ROOT.exists(),
            "state-machine implementation must not survive under core/framework",
        )
        self.assertTrue(KERNEL_STATE_ROOT.is_dir())
        self.assertEqual(
            REQUIRED_STATE_FILES,
            {path.name for path in KERNEL_STATE_ROOT.glob("*.rs")},
        )

        runtime_mod = (REPO_ROOT / "zircon_runtime/src/core/runtime/mod.rs").read_text(
            encoding="utf-8"
        )
        framework_mod = (
            REPO_ROOT / "zircon_runtime/src/core/framework/mod.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("pub mod state_machine;", runtime_mod)
        self.assertNotRegex(framework_mod, r"(?m)^\s*pub\s+mod\s+state\s*;")

    def test_product_rust_sources_do_not_consume_the_deleted_state_owner(self) -> None:
        violations = []
        for path in product_rust_candidate_paths():
            source = path.read_text(encoding="utf-8")
            source_lines = source.splitlines()
            for line_number in old_state_owner_lines(source):
                violations.append(
                    f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: "
                    f"{source_lines[line_number - 1].strip()}"
                )

        self.assertEqual(
            [],
            violations,
            "deleted core/framework/state consumers remain:\n" + "\n".join(violations),
        )

    def test_old_owner_scanner_handles_aliases_groups_and_qualified_paths(self) -> None:
        source = """
use crate::core::framework::{
    state::{NextState, State as RuntimeState},
    time::Time,
};

type Event = zircon_runtime
    :: core
    :: framework
    :: state
    :: StateTransitionEvent<Flow>;
"""

        self.assertEqual(old_state_owner_lines(source), [3, 7])

    def test_old_owner_scanner_ignores_comments_and_literals(self) -> None:
        source = r'''
// use crate::core::framework::state::State;
const DOC: &str = "zircon_runtime::core::framework::state";
/* crate::core::framework::state::NextState */
'''

        self.assertEqual(old_state_owner_lines(source), [])

    def test_old_owner_scanner_handles_chained_module_aliases(self) -> None:
        source = """
use crate::core as engine_core;
use engine_core::framework as contracts;

type Event = contracts::state::StateTransitionEvent<Flow>;
"""

        self.assertEqual(old_state_owner_lines(source), [5])

    def test_state_transition_observation_is_bounded_and_singular(self) -> None:
        machine = (KERNEL_STATE_ROOT / "machine.rs").read_text(encoding="utf-8")
        registry = (KERNEL_STATE_ROOT / "registry.rs").read_text(encoding="utf-8")
        handle = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/handle/states.rs"
        ).read_text(encoding="utf-8")
        runtime = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/runtime.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("latest_event: Option<StateTransitionEvent<T>>", machine)
        self.assertNotIn("events: Vec<StateTransitionEvent<T>>", machine)
        self.assertNotIn("self.events.push(", machine)
        self.assertIn("fn latest_event(&self)", machine)
        self.assertIn("fn latest_transition<T: StateSpec>", registry)
        self.assertIn("pub fn latest_state_transition<T", handle)
        self.assertIn("pub fn latest_state_transition<T", runtime)

        for source in (machine, registry, handle, runtime):
            self.assertNotIn("state_transition_events", source)
            self.assertNotIn("transition_events", source)


if __name__ == "__main__":
    unittest.main()
