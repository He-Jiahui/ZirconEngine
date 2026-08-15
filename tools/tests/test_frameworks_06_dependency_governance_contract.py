from __future__ import annotations

import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_TOOLCHAIN = "1.94.1"
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
        self.assertEqual(sources["allow-git"], [])

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
        self.assertEqual(job.count("uses: EmbarkStudios/cargo-deny-action@v2"), 1)
        self.assertNotRegex(
            job,
            r"(?m)^\s+(?:if|continue-on-error):",
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
        self.assertIn("uses: EmbarkStudios/cargo-deny-action@v2", action_step)
        self.assertNotRegex(
            action_step,
            r"(?m)^\s+(?:if|continue-on-error):",
        )
        self.assertIn(f'rust-version: "{RUST_TOOLCHAIN}"', action_step)
        self.assertIn("manifest-path: ${{ matrix.manifest }}", action_step)
        self.assertRegex(action_step, r"(?m)^\s+command:\s+check\s*$")
        self.assertIn(
            "command-arguments: advisories bans licenses sources",
            action_step,
        )
        self.assertIn("arguments: --all-features", action_step)
        for manifest in WORKSPACE_MANIFESTS:
            self.assertRegex(job, rf"(?m)^\s+- {re.escape(manifest)}\s*$")

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
                {f"registry+{registry}" for registry in allowed_registries},
                lock_path.as_posix(),
            )


if __name__ == "__main__":
    unittest.main()
