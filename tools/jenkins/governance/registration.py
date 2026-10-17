"""Bounded project registration from an immutable whole-volume observation."""
from __future__ import annotations

from collections import defaultdict
from pathlib import Path
import tomllib

from .paths import build_namespace, digest, file_fact, physical


def run(authority, payload, report):
    from .audit import _inventory
    from .worker import _json
    selected = _inventory(authority, payload["inventoryId"])
    if not selected:
        raise ValueError("original_completed_inventory_required")
    reference, observation = selected
    grouped = defaultdict(list)
    related = defaultdict(set)
    for item in observation["repositories"]:
        if item.get("commonDir"):
            grouped[item["commonDir"]].append(item)
            for root in item.get("rootCommits", []):
                related[root].add(item["commonDir"])
    existing = {row["payload"]["commonDir"]: row["payload"] for row in authority.state.list("governance_repository")}
    registered, review = [], []
    if len(grouped) > 4096:
        raise ValueError("repository_registration_budget_exceeded")
    for common, candidates in sorted(grouped.items()):
        if common in existing:
            registered.append({"repositoryId": existing[common]["repositoryId"], "canonicalCheckout": existing[common]["canonicalCheckout"],
                               "status": "existing-registration"})
            continue
        try:
            roots = {root for item in candidates for root in item.get("rootCommits", [])}
            if any(len(related[root]) > 1 for root in roots):
                raise ValueError("independent_histories_related_canonical_selection_requires_review")
            canonical = [item for item in candidates if not item.get("bare") and not item.get("errors")
                         and item.get("primaryBranch") and (Path(item["path"]) / ".git").is_dir()
                         and physical(Path(item["path"]) / ".git") == physical(common)]
            if len(canonical) != 1:
                raise ValueError("canonical_checkout_or_primary_not_unique")
            root = physical(canonical[0]["path"])
            from tools.jenkins.source import repository_id
            namespace = Path("E:/cargo-targets/zircon-local/governance") / repository_id(root)
            config = root / ".codex/config.toml"
            config_fact = None
            if config.exists():
                config_fact = file_fact(physical(config))
                settings = tomllib.loads(config.read_text(encoding="utf-8-sig"))
                target = settings.get("shell_environment_policy", {}).get("set", {}).get("CARGO_TARGET_DIR")
                if target:
                    namespace = build_namespace(target).parent
                if file_fact(config) != config_fact:
                    raise ValueError("project_configuration_changed_during_registration")
            recipes = payload.get("validationRecipesByCommonDir", {}).get(common, [])
            registration = authority.register({"canonicalCheckout": str(root), "primaryBranch": canonical[0]["primaryBranch"],
                                               "buildNamespace": str(build_namespace(namespace)), "validationRecipes": recipes})
            registered.append({"repositoryId": registration["repositoryId"], "canonicalCheckout": str(root),
                               "status": "registered", "configuration": config_fact,
                               "validation": "registered" if recipes else "needs-project-validation-recipe"})
        except (ValueError, OSError, tomllib.TOMLDecodeError) as error:
            review.append({"commonDir": common, "checkouts": [item["path"] for item in candidates], "status": "needs-review",
                           "reason": type(error).__name__ + ":" + str(error)[:200]})
    value = {"status": "registered-with-review" if review else "registered", "inventoryRef": reference["inventoryRef"],
             "inventoryDigest": reference["inventoryDigest"], "registered": registered, "needsReview": review,
             "sourceWritability": "not-proved-by-registration", **authority.runtime}
    _json(report / "repository-registration.json", value)
    authority.put("registration_batch", payload["operationId"], value, expected_version=0)
    return {"status": "needs-review" if review else "registered", "registeredCount": len(registered),
            "reviewCount": len(review), "report": file_fact(report / "repository-registration.json")}


def configure(authority, payload, report):
    registration = authority.get("repository", payload["repositoryId"])
    if digest(registration) != payload["registrationDigest"]:
        raise ValueError("original_repository_registration_changed")
    recipes = payload["validationRecipes"]
    if (not recipes or not isinstance(recipes, list) or len(recipes) > 32
            or any(not isinstance(item, dict) or not isinstance(item.get("argv"), list) or not item["argv"]
                   or not all(isinstance(arg, str) and "\x00" not in arg for arg in item["argv"]) for item in recipes)):
        raise ValueError("fixed_project_validation_recipes_required")
    with authority.state.transaction() as connection:
        row = authority.state.get("governance_repository", payload["repositoryId"], connection=connection)
        if digest(row["payload"]) != payload["registrationDigest"]:
            raise ValueError("repository_configuration_compare_and_swap_failed")
        completed = {row["payload"].get("commit") for row in authority.state.list("governance_integration_receipt", connection=connection)}
        for operation in authority.state.list("governance_integration", connection=connection):
            item = operation["payload"]
            if item["repositoryId"] == payload["repositoryId"] and item.get("status") == "validation-required" and item.get("commit") not in completed:
                raise ValueError("pending_integration_recipe_must_remain_immutable")
        value = {**registration, "validationRecipes": recipes}
        authority.put("repository", payload["repositoryId"], value, connection=connection, expected_version=row["version"])
        authority.put("repository_configuration", payload["operationId"], {"before": registration, "after": value,
                      "authorization": "explicit-user-governance-task", **authority.runtime}, connection=connection, expected_version=0)
    from .worker import _json
    _json(report / "repository-configuration.json", value)
    return {"status": "configured", "repositoryId": payload["repositoryId"], "registrationDigest": digest(value),
            "validationRecipesDigest": digest(recipes)}
