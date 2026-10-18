"""Git-index closure for the WOC Bot, Client, and Server source inputs."""

from __future__ import annotations

import json
import re
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "examples/woc/woc-source-manifest.json"
ZR_IMPORT = re.compile(r'%import\("([^"]+)"\)')
RUST_MODULE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z_0-9]*)\s*;",
    re.MULTILINE,
)


def git(*args: str, input: bytes | None = None) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["git", *args],
        cwd=ROOT,
        capture_output=True,
        check=False,
        input=input,
    )


def indexed_paths() -> set[str]:
    result = git("ls-files", "--cached", "-z")
    if result.returncode:
        raise AssertionError(result.stderr.decode("utf-8", errors="replace"))
    return {path.decode("utf-8") for path in result.stdout.split(b"\0") if path}


def ignored_paths(paths: set[str]) -> set[str]:
    payload = b"\0".join(path.encode("utf-8") for path in sorted(paths)) + b"\0"
    result = git("check-ignore", "--no-index", "-z", "--stdin", input=payload)
    if result.returncode not in (0, 1):
        raise AssertionError(result.stderr.decode("utf-8", errors="replace"))
    return {path.decode("utf-8") for path in result.stdout.split(b"\0") if path}


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def required_paths(manifest: dict) -> set[str]:
    required = {relative(MANIFEST)}
    for group in (
        "entrypoints",
        "build_inputs",
        "product_sources",
        "test_sources",
        "producers",
        "references",
    ):
        required.update(manifest[group])
    for dependency in manifest["generated_dependencies"]:
        required.add(dependency["producer"])
        required.update(dependency["inputs"])
        required.update(dependency["outputs"])
    return required


def zr_closure() -> tuple[set[str], set[str]]:
    source_root = ROOT / "examples/woc/scripts/woc_game/src"
    pending = ["main"]
    visited: set[str] = set()
    source_paths: set[str] = set()
    absent: set[str] = set()
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        visited.add(module)
        path = source_root / f"{module}.zr"
        if not path.is_file():
            absent.add(relative(path))
            continue
        source_paths.add(relative(path))
        for dependency in ZR_IMPORT.findall(path.read_text(encoding="utf-8")):
            if not dependency.startswith("zr."):
                pending.append(dependency)
    return source_paths, absent


def rust_closure() -> tuple[set[str], set[str]]:
    roots = [
        "examples/woc/native/crates/woc_protocol/src/lib.rs",
        "examples/woc/native/plugins/woc_runtime/src/lib.rs",
        "examples/woc/native/apps/woc_server/src/lib.rs",
        "examples/woc/native/apps/woc_client/src/lib.rs",
        "examples/woc/native/apps/woc_bot/src/main.rs",
    ]
    pending = [ROOT / root for root in roots]
    visited: set[str] = set()
    absent: set[str] = set()
    while pending:
        path = pending.pop()
        name = relative(path)
        if name in visited:
            continue
        visited.add(name)
        if not path.is_file():
            absent.add(name)
            continue
        module_root = (
            path.parent
            if path.name in ("lib.rs", "main.rs", "mod.rs")
            else path.parent / path.stem
        )
        for module in RUST_MODULE.findall(path.read_text(encoding="utf-8")):
            file_module = module_root / f"{module}.rs"
            directory_module = module_root / module / "mod.rs"
            pending.append(file_module if file_module.is_file() else directory_module)
    return visited, absent


class WocSourceClosureTest(unittest.TestCase):
    def manifest(self) -> dict:
        return json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_product_source_graph_is_declared_and_tracked(self) -> None:
        manifest = self.manifest()
        indexed = indexed_paths()
        zr, absent_zr = zr_closure()
        rust, absent_rust = rust_closure()
        self.assertEqual([], sorted(absent_zr | absent_rust))

        declared = set(manifest["product_sources"])
        graph = zr | rust
        self.assertEqual(
            [],
            sorted(path for path in graph - indexed if path not in declared),
            "new untracked imports must be classified in the WOC source manifest",
        )
        generated_sources = {
            output
            for dependency in manifest["generated_dependencies"]
            for output in dependency["outputs"]
            if output.endswith((".rs", ".zr"))
        }
        self.assertEqual(
            [],
            sorted(generated_sources - declared),
            "generated Rust and Zr modules must be classified as product sources",
        )
        required = required_paths(manifest)
        self.assertEqual([], sorted(required - indexed), "clean clone omits WOC inputs")

    def test_generated_dependencies_cover_scene_models_and_producers(self) -> None:
        manifest = self.manifest()
        dependencies = manifest["generated_dependencies"]
        outputs = {output for dependency in dependencies for output in dependency["outputs"]}
        scene = ROOT / "examples/woc/assets/scenes/eastbrook_mvp.scene.toml"
        model_refs = {
            f"examples/woc/{path}"
            for path in re.findall(
                r'path_hint = "(assets/models/[^\"]+\.model\.toml)"',
                scene.read_text(encoding="utf-8"),
            )
        }
        self.assertEqual([], sorted(model_refs - outputs), "generated scene model inputs are undeclared")

        for dependency in dependencies:
            producer = dependency["producer"]
            self.assertIn(producer, manifest["producers"])
            for output in dependency["outputs"]:
                path = ROOT / output
                self.assertTrue(path.is_file(), f"generated dependency is absent: {output}")
                text = path.read_text(encoding="utf-8")
                match = re.search(r"Generated by (tools/[\w.-]+\.mjs)", text[:200])
                if match:
                    self.assertEqual(producer, f"examples/woc/{match.group(1)}", output)
                elif output.endswith(".json"):
                    self.assertEqual(
                        producer,
                        json.loads(text).get("generated_by", producer),
                        output,
                    )

    def test_required_sources_are_visible_without_exposing_artifacts(self) -> None:
        manifest = self.manifest()
        indexed = indexed_paths()
        required = required_paths(manifest)
        ignored = sorted(ignored_paths(required - indexed))
        self.assertEqual([], ignored, "WOC source inputs remain ignored")
        for artifact in (
            "examples/woc/native/target/debug/woc_server.exe",
            "examples/woc/scripts/woc_game/build/main.zro",
            "examples/woc/assets/generated/cooked.bin",
        ):
            self.assertEqual(
                0,
                git("check-ignore", "--no-index", "-q", "--", artifact).returncode,
                artifact,
            )

    def test_compiled_zro_outputs_are_not_indexed(self) -> None:
        compiled = sorted(
            path
            for path in indexed_paths()
            if path.startswith("examples/woc/scripts/woc_game/bin")
            and path.endswith(".zro")
        )
        self.assertEqual(
            [],
            compiled,
            "compiled .zro files remain in the Git index; integration must remove them",
        )


if __name__ == "__main__":
    unittest.main()
