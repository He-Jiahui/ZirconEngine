"""Jenkins validation of the exact integration tree and focused governance tests."""
from __future__ import annotations

import os
from pathlib import Path
import sys

from .paths import build_namespace, digest, file_fact, physical


def run(authority, payload, report, *, self_test=False):
    from .worker import _json, _terminal, native, register_artifacts
    if self_test:
        import base64
        import json
        import zlib
        pins = json.loads(zlib.decompress(base64.b64decode(os.environ["ZGOV_PINS"], validate=True)))
        def changed_inputs():
            return [name for name, expected in pins["sourcePins"].items()
                    if file_fact(physical(authority.root / name))["sha256"] != expected]
        if changed_inputs():
            raise ValueError("functional_test_source_changed_before_execution")
        fixture = report / "fixtures"
        fixture.mkdir()
        selected = payload.get("testNames", [])
        if not isinstance(selected, list) or any(not isinstance(name, str) or not name.startswith("test_") for name in selected):
            raise ValueError("fixed_governance_test_names_required")
        result = native(authority, [sys.executable, "-B", "-X", "utf8", "-m", "tools.jenkins.governance.tests"],
                        authority.root, report / "native-tests", timeout=1500 if selected else 3600,
                        environment={"ZGOV_TEST_ROOT": str(fixture), "ZGOV_TEST_NAMES": json.dumps(selected)})
        changed = changed_inputs()
        success = _terminal(result) and result.get("rootExitCode") == 0 and not changed
        cleanup = None
        if success:
            from .fixtures import remove
            cleanup = remove(authority, report)
        value = {"status": ("passed-scoped" if selected else "passed") if success else "failed", "nativeReceipt": file_fact(report / "native-tests/native-receipt.json"),
                 "capsuleDigest": authority.runtime["capsuleDigest"], "driverDigest": authority.runtime["driverDigest"],
                 "fixtureRoot": str(fixture), "fixtureCleanup": cleanup, "scope": "governance-contracts", "testNames": selected,
                 "permissionAcceptance": False, "changedSourceInputs": changed, "sourceInputsUnchanged": not changed}
        _json(report / "test-summary.json", value)
        authority.put("scoped_test" if selected else "capability",
                      payload["operationId"] if selected else authority.runtime["capsuleDigest"], value, expected_version=0)
        return value
    authority.require_closed(payload)
    integration = authority.get("integration", payload["integrationOperationId"])
    if integration["status"] != "validation-required" or integration["repositoryId"] != payload["repositoryId"]:
        raise ValueError("integration_validation_input_invalid")
    registration = authority.get("repository", payload["repositoryId"])
    recipes = registration["validationRecipes"]
    if not recipes:
        return {"status": "needs-review", "reason": "project_validation_recipe_not_registered"}
    # Jenkins prepares a single reusable detached checkout of this exact tree.
    # The caller cannot inject another source root or arbitrary command here.
    checkout = build_namespace(registration["buildNamespace"]) / "governance-validation-source"
    register_artifacts(authority, [
        {"path": str(checkout), "role": "Jenkins-prepared-validation-source", "retainUntil": "exact validation is terminal and worktree metadata is reconciled"},
        {"path": str(build_namespace(registration["buildNamespace"])), "role": "Jenkins-compiler-output-and-reuse-pool", "retainUntil": "registered retention released after writer termination"}])
    from .versioning import write_git
    from .inventory import repository_fact, text_git
    canonical = physical(registration["canonicalCheckout"])
    if not checkout.exists():
        checkout.parent.mkdir(parents=True, exist_ok=True)
        write_git(canonical, ["worktree", "add", "--detach", str(checkout), integration["commit"]])
    else:
        observation = repository_fact(checkout)
        if observation.get("errors") or observation.get("changes"):
            raise ValueError("reusable_validation_checkout_requires_reconciliation")
        write_git(checkout, ["checkout", "--detach", integration["commit"]])
    if text_git(checkout, "rev-parse", "HEAD^{tree}") != integration["resultTree"]:
        raise ValueError("prepared_validation_tree_mismatch")
    receipts, passed = [], True
    build = build_namespace(registration["buildNamespace"])
    for index, recipe in enumerate(recipes):
        substitutions = {"{checkout}": str(checkout), "{buildRoot}": str(build)}
        argv = [substitutions.get(arg, arg) for arg in recipe["argv"]]
        # guard-allow(storage.output-root): build_namespace verifies a physical drive-root D/E/F cargo-targets namespace.
        env = {"CARGO_TARGET_DIR": str(build / "target"), "CARGO_BUILD_BUILD_DIR": str(build / "build"),
               "PYTHONDONTWRITEBYTECODE": "1", "CARGO_INCREMENTAL": "0"}
        evidence = native(authority, argv, checkout, report / f"validation-{index}",
                          timeout=min(float(recipe.get("timeoutSeconds", 1800)), 7200), environment=env)
        receipts.append(file_fact(report / f"validation-{index}" / "native-receipt.json"))
        if not _terminal(evidence) or evidence.get("rootExitCode") != 0:
            passed = False
            break
    after = repository_fact(checkout)
    if after.get("head") != integration["commit"] or after.get("changes") or after.get("errors"):
        passed = False
    value = {"status": "passed" if passed else "failed", "repositoryId": payload["repositoryId"],
             "integrationOperationId": payload["integrationOperationId"], "resultTree": integration["resultTree"],
             "commit": integration["commit"], "validationRecipesDigest": digest(recipes), "nativeReceipts": receipts,
             "validationCheckout": str(checkout), **authority.runtime}
    authority.put("validation", payload["operationId"], value, expected_version=0)
    return value
