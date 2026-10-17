from __future__ import annotations

import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_TOOLCHAIN = "1.94.1"
CARGO_DENY_VERSION = "0.20.2"
WORKSPACE_MANIFESTS = ("Cargo.toml", "zircon_plugins/Cargo.toml")
GLOBAL_LICENSE_ALLOW = {
    "0BSD",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "BSL-1.0",
    "CC0-1.0",
    "ISC",
    "MIT",
    "MIT-0",
    "NCSA",
    "Unicode-3.0",
    "Unlicense",
    "Zlib",
}
PER_CRATE_MPL_EXCEPTIONS = {
    "audio_thread_priority",
    "cssparser",
    "cssparser-macros",
    "dtoa-short",
    "option-ext",
    "selectors",
    "symphonia",
    "symphonia-bundle-flac",
    "symphonia-bundle-mp3",
    "symphonia-codec-pcm",
    "symphonia-codec-vorbis",
    "symphonia-common",
    "symphonia-core",
    "symphonia-format-ogg",
    "symphonia-format-riff",
    "symphonia-metadata",
    "symphonia-utils-xiph",
    "triple_buffer",
}

# 解析依赖策略、锁文件和 CI 工作流，确认来源默认拒绝、许可证例外与多工作区检查一致。
class Frameworks06DependencyGovernanceContractTests(unittest.TestCase):
    def test_dependency_policy_is_explicit_and_deny_by_default_for_sources(self) -> None:
        policy_path = REPO_ROOT / "deny.toml"
        self.assertTrue(policy_path.is_file(), "workspace dependency policy must exist")
        with policy_path.open("rb") as source:
            policy = tomllib.load(source)

        advisories = policy["advisories"]
        self.assertEqual(advisories["ignore"], [])
        self.assertEqual(set(advisories), {"ignore"})

        bans = policy["bans"]
        self.assertEqual(bans["multiple-versions"], "warn")
        self.assertEqual(bans["wildcards"], "deny")
        self.assertEqual(bans["deny"], [])
        self.assertEqual(bans["skip"], [])
        self.assertEqual(bans["skip-tree"], [])

        sources = policy["sources"]
        self.assertEqual(sources["unknown-registry"], "deny")
        self.assertEqual(sources["unknown-git"], "deny")
        self.assertEqual(
            sources["allow-registry"],
            ["https://github.com/rust-lang/crates.io-index"],
        )
        self.assertEqual(sources["allow-git"], ["https://github.com/He-Jiahui/zr_vm.git"])
        self.assertEqual(sources["required-git-spec"], "rev")

    def test_copyleft_license_is_scoped_to_existing_crates(self) -> None:
        with (REPO_ROOT / "deny.toml").open("rb") as source:
            licenses = tomllib.load(source)["licenses"]

        self.assertEqual(set(licenses["allow"]), GLOBAL_LICENSE_ALLOW)
        self.assertEqual(len(licenses["allow"]), len(GLOBAL_LICENSE_ALLOW))
        self.assertEqual(licenses["confidence-threshold"], 0.8)
        self.assertEqual(
            licenses["private"],
            {"ignore": False, "registries": []},
        )
        mpl_exceptions = {
            entry["crate"]
            for entry in licenses["exceptions"]
            if entry["allow"] == ["MPL-2.0"]
        }
        self.assertEqual(mpl_exceptions, PER_CRATE_MPL_EXCEPTIONS)
        self.assertEqual(len(mpl_exceptions), len(licenses["exceptions"]))

    def test_ci_checks_root_and_plugin_workspaces_with_named_toolchain(self) -> None:
        workflow = (REPO_ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        job_match = re.search(
            r"(?ms)^  dependency-governance:\s*$\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:\s*$|\Z)",
            workflow,
        )
        self.assertIsNotNone(job_match, "CI must own a dependency-governance job")
        job = job_match.group("body")

        self.assertEqual(
            job.count(
                "python -B -m unittest "
                "tools.tests.test_frameworks_06_dependency_governance_contract -v"
            ),
            1,
        )
        self.assertIn("runs-on: [self-hosted, Windows, X64]", job)
        self.assertIn("uses: dtolnay/rust-toolchain@75be91dd2711b583df57c31d0873b4145c89f1d9", job)
        self.assertIn(f"toolchain: {RUST_TOOLCHAIN}", job)
        self.assertIn("run: ./.github/scripts/prepare-cargo-roots.ps1", job)
        self.assertIn(
            r"CARGO_TARGET_DIR: E:\cargo-targets\zircon-local\ci\runs\${{ github.run_id }}-${{ github.run_attempt }}\dependency-governance-${{ strategy.job-index || 0 }}\target",
            job,
        )
        for manifest, output in zip(WORKSPACE_MANIFESTS, ("root", "plugins")):
            self.assertRegex(
                job,
                rf"(?m)^\s+- manifest: {re.escape(manifest)}\s*\n\s+output: {output}\s*$",
            )
        self.assertNotRegex(
            job,
            r"(?m)^ {6,}(?:if|continue-on-error):",
        )
        install_step_match = re.search(
            r"(?ms)^      - name: Install pinned cargo-deny\s*$\n"
            r"(?P<step>.*?)(?=^      - |\Z)",
            job,
        )
        self.assertIsNotNone(install_step_match, "CI must pin the native cargo-deny binary")
        install_step = install_step_match.group("step")
        self.assertIn("shell: pwsh", install_step)
        self.assertIn(
            f"./.github/scripts/invoke-cargo-msvc.ps1 cargo +{RUST_TOOLCHAIN} "
            f"install --locked --version {CARGO_DENY_VERSION} "
            '--root "$env:CARGO_TARGET_DIR\\cargo-deny" cargo-deny',
            install_step,
        )
        action_step_match = re.search(
            r"(?ms)^      - name: Check advisories, duplicate versions, licenses, and sources\s*$\n"
            r"(?P<step>.*?)(?=^      - |\Z)",
            job,
        )
        self.assertIsNotNone(
            action_step_match,
            "CI must keep dependency-governance inputs on the cargo-deny step",
        )
        action_step = action_step_match.group("step")
        self.assertIn("shell: pwsh", action_step)
        self.assertNotRegex(
            action_step,
            r"(?m)^ {6,}(?:if|continue-on-error):",
        )
        self.assertIn(
            './.github/scripts/invoke-cargo-msvc.ps1 '
            '"$env:CARGO_TARGET_DIR\\cargo-deny\\bin\\cargo-deny.exe" '
            '--manifest-path "${{ matrix.manifest }}" '
            '--all-features check advisories bans licenses sources',
            action_step,
        )
        for manifest in WORKSPACE_MANIFESTS:
            self.assertRegex(job, rf"(?m)^\s+- {re.escape(manifest)}\s*$")

    def test_vm_sources_are_consistent_without_a_sibling_checkout(self) -> None:
        bindings = ("zr_vm_rust_binding", "zr_vm_rust_binding_sys")
        plugin = tomllib.loads((REPO_ROOT / "zircon_plugins/zr_vm_language/runtime/Cargo.toml").read_text())["dependencies"]
        example = tomllib.loads((REPO_ROOT / "examples/woc/native/Cargo.toml").read_text())["workspace"]["dependencies"]
        expected = {"git": "https://github.com/He-Jiahui/zr_vm.git", "rev": "3c1352ecb6ddfab02752d46682cc376558d5f481"}
        for dependencies in (plugin, example):
            for name in bindings:
                self.assertNotIn("path", dependencies[name])
                self.assertEqual({key: dependencies[name][key] for key in expected}, expected)
        source = "git+" + expected["git"] + "?rev=" + expected["rev"] + "#" + expected["rev"]
        for path in ("Cargo.lock", "zircon_plugins/Cargo.lock", "examples/woc/native/Cargo.lock"):
            packages = tomllib.loads((REPO_ROOT / path).read_text())["package"]
            self.assertEqual([p.get("source") for p in packages if p["name"] in bindings], [source, source])

    def test_lockfiles_use_only_sources_authorized_by_policy(self) -> None:
        with (REPO_ROOT / "deny.toml").open("rb") as source:
            allowed_registries = set(tomllib.load(source)["sources"]["allow-registry"])

        for manifest in WORKSPACE_MANIFESTS:
            lock_path = (REPO_ROOT / manifest).with_name("Cargo.lock")
            with lock_path.open("rb") as source:
                packages = tomllib.load(source)["package"]
            external_sources = {
                package["source"] for package in packages if "source" in package
            }
            self.assertEqual(
                external_sources,
                {f"registry+{registry}" for registry in allowed_registries} | {
                    "git+https://github.com/He-Jiahui/zr_vm.git?rev=3c1352ecb6ddfab02752d46682cc376558d5f481#3c1352ecb6ddfab02752d46682cc376558d5f481"
                },
                lock_path.as_posix(),
            )


if __name__ == "__main__":
    unittest.main()
