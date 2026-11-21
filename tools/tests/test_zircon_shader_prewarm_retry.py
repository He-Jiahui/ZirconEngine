"""Black-box retry checks for the ordinary zircon_shader_prewarm executable.

Set ZR_TEST_SHADER_PREWARM_EXE to a built CLI. The migration cases also need
ZR_TEST_SHADER_PREWARM_V4_EXE from before the retry fix. Set
ZR_TEST_SHADER_PREWARM_RETRY_OUTPUT_DIR to retain CLI reports and logs.
"""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


EXE_VAR = "ZR_TEST_SHADER_PREWARM_EXE"
LEGACY_EXE_VAR = "ZR_TEST_SHADER_PREWARM_V4_EXE"
OUTPUT_VAR = "ZR_TEST_SHADER_PREWARM_RETRY_OUTPUT_DIR"
SHADER_GUID = "00000000-0000-4000-8000-000000000219"
SECOND_GUID = "00000000-0000-4000-8000-000000000220"
LEGAL_BITS = {0, 0x40000000, 0x80000000}
BROKEN_MATERIAL = "[options\n"
WGSL = """fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(vec4<f32>(input.color.rgb, 1.0));
}
"""


def write_source(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def author_project_assets(project, name="budget", guid=SHADER_GUID):
    assets = project / "Assets"
    shader = (
        'version = 2\nkind = "surface"\nshading_model = "standard_pbr"\n'
        'wgsl_files = ["surface.wgsl"]\n'
        'disabled_passes = ["gbuffer", "depth_prepass", "shadow", "velocity", "taa_reactive_mask"]\n'
    )
    for i in range(30):
        shader += f'\n[[options]]\nname = "flag_{i:02d}"\nkind = "bool"\ndefault = false\n'
    shader += (
        '\n[[options]]\nname = "detail_mode"\nkind = "enum"\ndefault = "off"\n'
        'editor = { values = "off,uv,triplanar" }\n'
    )
    shader_dir = assets / "shaders" / name
    write_source(shader_dir / f"{name}.zshader", shader)
    # Distinct WGSL for independently authored roots so source dedupe cannot
    # substitute one root's provenance for the other in a combined report.
    wgsl = WGSL if name == "budget" else WGSL.replace("1.0", "0.75")
    write_source(shader_dir / "surface.wgsl", wgsl)
    digest = hashlib.sha256((shader + wgsl).encode("utf-8")).hexdigest()
    write_source(
        shader_dir.with_suffix(".zmeta"),
        'format_version = 7\nasset_kind = "Shader"\nunit = "compound"\n'
        f'uuid = "{guid}"\nurl = "res://shaders/{name}"\nsource_digest = "{digest}"\n',
    )
    for i, value in enumerate(("off", "uv", "triplanar", "triplanar")):
        material_name = f"detail_{i}_{value}"
        write_source(
            assets / "materials" / f"{material_name}.zmaterial",
            f'version = 2\nname = "{material_name}"\n'
            f'\n[shader]\nkind = "project"\nguid = "{guid}"\n'
            f'path_hint = "Assets/shaders/{name}"\n'
            '\n[overrides]\nreceive_shadows = false\n'
            f'\n[options]\ndetail_mode = "{value}"\n',
        )
    return assets


def authored_file_hashes(roots):
    return {
        str(path): hashlib.sha256(path.read_bytes()).hexdigest()
        for root in roots
        for path in root.rglob("*")
        if path.is_file()
    }


class ZirconShaderPrewarmRetryTests(unittest.TestCase):
    def setUp(self):
        self.exe = self.require_executable(EXE_VAR)
        output = os.environ.get(OUTPUT_VAR)
        if output:
            self.output = Path(output).resolve() / self._testMethodName
            self.output.mkdir(parents=True, exist_ok=False)
        else:
            self.output = Path(self.enterContext(tempfile.TemporaryDirectory(prefix="zr-prewarm-")))

    def require_executable(self, variable):
        value = os.environ.get(variable)
        if not value:
            self.skipTest(f"set {variable} to an ordinary zircon_shader_prewarm executable")
        exe = Path(value).resolve()
        self.assertTrue(exe.is_file(), f"{variable} does not name a file: {exe}")
        return exe

    def invoke(self, case, label, roots, *options, exe=None):
        directory = self.output / case
        report_path = directory / f"{label}.report.json"
        cache = directory / "cache"
        temp = directory / "tmp"
        temp.mkdir(parents=True, exist_ok=True)
        command = [
            str(exe or self.exe), "--project-root", str(directory), "--cache-dir", str(cache),
        ]
        for root in roots:
            command.extend(("--asset-root", str(root)))
        command.extend(("--report", str(report_path), *options))
        before = authored_file_hashes(roots)
        result = subprocess.run(
            command, cwd=directory, capture_output=True, check=False, timeout=90,
            env=os.environ | {name: str(temp) for name in ("TEMP", "TMP", "TMPDIR")},
        )
        (directory / f"{label}.stdout").write_bytes(result.stdout)
        (directory / f"{label}.stderr").write_bytes(result.stderr)
        observed = {
            "command": command,
            "exit": result.returncode,
            "stderr": result.stderr.decode("utf-8", errors="replace"),
            "report": str(report_path) if report_path.is_file() else None,
            "assets_unchanged": before == authored_file_hashes(roots),
        }
        write_source(directory / f"{label}.observation.json", json.dumps(observed, indent=2) + "\n")
        self.assertTrue(observed["assets_unchanged"], observed)
        return observed

    def assert_parse_error(self, observed, material="invalid.zmaterial"):
        self.assertEqual((1, None), (observed["exit"], observed["report"]), observed)
        for text in ("failed to parse zmaterial", material, "unclosed table"):
            self.assertIn(text, observed["stderr"], observed)

    def assert_report(self, observed, count):
        self.assertEqual(0, observed["exit"], observed)
        self.assertIsNotNone(observed["report"], observed)
        report = json.loads(Path(observed["report"]).read_bytes())
        counts = tuple(report[f"{name}_count"] for name in ("requested", "written", "failed"))
        self.assertEqual((count, count, 0), counts, report)
        self.assertIsNone(report.get("preflight_error"), report)
        return report

    def assert_legal_variants(self, observed, name="budget"):
        report = self.assert_report(observed, len(LEGAL_BITS))
        rows = report["written_variants"]
        self.assertEqual(len(LEGAL_BITS), len(rows), rows)
        self.assertEqual({f"res://shaders/{name}"}, {row["source_label"] for row in rows})
        options = {
            int(field.removeprefix("material_options="), 16)
            for row in rows
            for field in row["canonical_string"].split("|")
            if field.startswith("material_options=")
        }
        self.assertEqual(LEGAL_BITS, options)

    def test_same_bad_material_fails_on_default_retry(self):
        case = "broken"
        roots = (author_project_assets(self.output / case),)
        write_source(roots[0] / "materials/invalid.zmaterial", BROKEN_MATERIAL)
        self.assert_parse_error(self.invoke(case, "cold", roots))
        self.assert_parse_error(self.invoke(case, "warm", roots))

    def test_successful_legal_variants_keep_the_default_fast_path(self):
        case = "valid32"
        roots = (author_project_assets(self.output / case),)
        self.assert_legal_variants(self.invoke(case, "cold", roots))
        self.assert_report(self.invoke(case, "warm", roots), 0)
        self.assert_legal_variants(self.invoke(case, "forced", roots, "--quality-tier", "high"))
        self.assert_report(self.invoke(case, "after-forced", roots), 0)

    def test_edit_after_success_is_rejected_on_default_retry(self):
        case = "edit"
        roots = (author_project_assets(self.output / case),)
        self.assert_legal_variants(self.invoke(case, "valid", roots))
        edited = roots[0] / "materials/detail_2_triplanar.zmaterial"
        write_source(edited, BROKEN_MATERIAL)
        for label in ("cold-error", "warm-error"):
            self.assert_parse_error(self.invoke(case, label, roots), edited.name)

    def test_failure_in_later_root_leaves_earlier_root_pending(self):
        case = "partial"
        project = self.output / case
        good = author_project_assets(project / "a")
        bad = author_project_assets(project / "b", "other", SECOND_GUID)
        invalid = bad / "materials/invalid.zmaterial"
        write_source(invalid, BROKEN_MATERIAL)
        roots = (good, bad)
        self.assert_parse_error(self.invoke(case, "cold", roots))
        self.assert_parse_error(self.invoke(case, "warm", roots))
        self.assert_legal_variants(self.invoke(case, "good-only", (good,)))
        invalid.write_bytes((invalid.parent / "detail_2_triplanar.zmaterial").read_bytes())
        self.assert_legal_variants(self.invoke(case, "repaired", roots), "other")
        self.assert_report(self.invoke(case, "repaired-warm", roots), 0)

    def test_v4_success_is_reprojected_and_the_v4_reader_still_works(self):
        legacy = self.require_executable(LEGACY_EXE_VAR)
        self.assertNotEqual(self.exe, legacy)
        case = "v4-success"
        roots = (author_project_assets(self.output / case),)
        self.assert_legal_variants(self.invoke(case, "seed", roots, exe=legacy))
        self.assert_legal_variants(self.invoke(case, "upgrade", roots))
        self.assert_report(self.invoke(case, "upgraded-warm", roots), 0)
        export = self.output / case / "legacy-registry.json"
        self.assert_report(
            self.invoke(case, "legacy-export", roots, "--export-resource-registry", str(export), exe=legacy),
            0,
        )
        records = json.loads(export.read_bytes())["resources"]
        self.assertEqual(["res://shaders/budget"], [row["primary_locator"] for row in records])

    def test_v4_failure_has_no_retry_eligibility_after_upgrade(self):
        legacy = self.require_executable(LEGACY_EXE_VAR)
        self.assertNotEqual(self.exe, legacy)
        case = "v4-failure"
        roots = (author_project_assets(self.output / case),)
        write_source(roots[0] / "materials/invalid.zmaterial", BROKEN_MATERIAL)
        self.assert_parse_error(self.invoke(case, "seed", roots, exe=legacy))
        self.assert_parse_error(self.invoke(case, "upgrade", roots))
        self.assert_parse_error(self.invoke(case, "upgrade-retry", roots))


if __name__ == "__main__":
    unittest.main()
