"""Transactional repository and claim authority in the existing Jenkins State."""
from __future__ import annotations

import time
import re
import json
from pathlib import Path

from .inventory import repository_fact, text_git
from .paths import build_namespace, digest, file_fact, overlaps, physical, relative_source


class Governance:
    def __init__(self, state, control_root: Path, runtime: dict):
        self.state, self.root, self.runtime = state, physical(control_root), dict(runtime)

    def get(self, domain: str, key: str, *, connection=None) -> dict:
        row = self.state.get("governance_" + domain, key, connection=connection)
        if row is None:
            raise ValueError("governance_record_missing:" + domain)
        return row["payload"]

    def put(self, domain: str, key: str, value: dict, *, connection=None, expected_version=None):
        return self.state.put("governance_" + domain, key, value,
                              connection=connection, expected_version=expected_version)

    def register(self, payload: dict) -> dict:
        root = physical(payload["canonicalCheckout"])
        fact = repository_fact(root)
        if fact["errors"] or fact.get("bare") or fact.get("primaryBranch") is None and not payload.get("primaryBranch"):
            raise ValueError("repository_registration_needs_review")
        primary = payload.get("primaryBranch") or fact["primaryBranch"]
        if "refs/heads/" + primary not in [branch["ref"] for branch in fact["branches"]]:
            raise ValueError("primary_branch_missing")
        # Preserve the source authority's existing physical-root repository identity.
        from tools.jenkins.source import repository_id
        identity = repository_id(root)
        build = build_namespace(payload["buildNamespace"])
        validations = payload.get("validationRecipes", [])
        if (not isinstance(validations, list) or any(not isinstance(item, dict)
                or not isinstance(item.get("argv"), list) or not item["argv"]
                or not all(isinstance(arg, str) and "\x00" not in arg for arg in item["argv"])
                for item in validations)):
            raise ValueError("invalid_validation_recipe")
        value = {"repositoryId": identity, "canonicalCheckout": str(root), "commonDir": fact["commonDir"],
                 "primaryBranch": primary, "buildNamespace": str(build), "validationRecipes": validations,
                 "driverDigest": self.runtime["driverDigest"], "registeredAt": time.time(),
                 "initialHead": fact["head"], "sourceIdentity": fact["repositoryId"],
                 "extraCheckouts": payload.get("extraCheckouts", []), "status": "registered"}
        with self.state.transaction() as connection:
            prior = self.state.get("governance_repository", identity, connection=connection)
            if prior:
                old = prior["payload"]
                keys = ["canonicalCheckout", "commonDir", "primaryBranch", "buildNamespace", "validationRecipes", "extraCheckouts"]
                if any(old.get(key) != value.get(key) for key in keys):
                    raise ValueError("repository_registration_changed")
                return old
            for row in self.state.list("governance_repository", connection=connection):
                if row["payload"]["commonDir"] == value["commonDir"]:
                    raise ValueError("repository_already_registered")
            self.put("repository", identity, value, connection=connection, expected_version=0)
        return value

    def open_round(self, payload: dict) -> dict:
        inventory = payload["inventoryRef"]
        physical(inventory["path"])
        if file_fact(Path(inventory["path"]))["sha256"] != inventory["sha256"]:
            raise ValueError("inventory_reference_changed")
        observation = json.loads(Path(inventory["path"]).read_text(encoding="utf-8"))
        if observation.get("inventoryDigest") != payload["inventoryDigest"]:
            raise ValueError("inventory_digest_mismatch")
        if digest({k: v for k, v in observation.items() if k != "inventoryDigest"}) != payload["inventoryDigest"]:
            raise ValueError("inventory_content_mismatch")
        value = {"roundId": payload["roundId"], "inventoryRef": inventory,
                 "inventoryDigest": payload["inventoryDigest"], "status": "awaiting-user-claims",
                 "createdAt": time.time(), **self.runtime}
        with self.state.transaction() as connection:
            prior = self.state.get("governance_round", value["roundId"], connection=connection)
            if prior:
                if prior["payload"]["inventoryDigest"] != value["inventoryDigest"]:
                    raise ValueError("claim_round_inventory_changed")
                return prior["payload"]
            self.put("round", value["roundId"], value, connection=connection, expected_version=0)
        return value

    def claim(self, payload: dict) -> dict:
        repository = self.get("repository", payload["repositoryId"])
        root = physical(payload.get("sourceCheckout", repository["canonicalCheckout"]))
        observation = repository_fact(root)
        if observation.get("commonDir") != repository["commonDir"]:
            raise ValueError("claim_checkout_repository_mismatch")
        selected = sorted({relative_source(value) for value in payload["paths"]})
        if not selected:
            raise ValueError("claim_paths_required")
        session = payload["ownerSessionId"]
        if not isinstance(session, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}", session):
            raise ValueError("claim_owner_required")
        from .scopes import inspect
        inputs = inspect(root, selected)
        hashes = inputs["hashes"]
        if payload.get("expectedHashes") != hashes:
            raise ValueError("claim_source_changed")
        key = digest({"round": payload["roundId"], "repository": payload["repositoryId"], "owner": session})
        value = {"claimId": key, "roundId": payload["roundId"], "repositoryId": payload["repositoryId"],
                 "ownerSessionId": session, "paths": selected, "hashes": hashes,
                 "sourceHashes": {path: fact["sha256"] if fact else None for path, fact in inputs["sourceFacts"].items()},
                 "sourceCheckout": str(root),
                 "epoch": 1, "status": "claimed", "baseHead": text_git(root, "rev-parse", "HEAD"),
                 "branch": "codex/jenkins/" + session + "/" + payload["roundId"], **self.runtime}
        with self.state.transaction() as connection:
            round_value = self.get("round", payload["roundId"], connection=connection)
            if round_value["status"] != "awaiting-user-claims":
                raise ValueError("claim_round_closed")
            old = self.state.get("governance_claim", key, connection=connection)
            if old:
                stable = ["repositoryId", "ownerSessionId", "paths", "hashes", "sourceHashes", "roundId", "sourceCheckout"]
                if any(old["payload"].get(k) != value.get(k) for k in stable):
                    raise ValueError("claim_changed")
                return old["payload"]
            for row in self.state.list("governance_claim", connection=connection):
                other = row["payload"]
                if other["repositoryId"] == value["repositoryId"] and other["status"] == "claimed":
                    if any(overlaps(a, b) for a in selected for b in other["paths"]):
                        raise ValueError("claim_path_overlap")
            for row in self.state.list("path_claims", connection=connection):
                other = row["payload"]
                if (other.get("repositoryId") == value["repositoryId"]
                        and other.get("owner") != session
                        and any(overlaps(path, other["path"]) for path in selected)):
                    raise ValueError("existing_source_claim_requires_reconciliation")
            self.put("claim", key, value, connection=connection, expected_version=0)
        return value

    def record_response(self, payload: dict) -> dict:
        reply = payload.get("userReply")
        if not isinstance(reply, str) or not reply.strip():
            raise ValueError("explicit_user_reply_required")
        with self.state.transaction() as connection:
            current = self.get("round", payload["roundId"], connection=connection)
            if current["status"] != "awaiting-user-claims":
                raise ValueError("claim_round_closed")
            prior = self.state.get("governance_response", payload["roundId"], connection=connection)
            value = {"roundId": payload["roundId"], "userReply": reply,
                     "replyDigest": digest(reply), "origin": "operator-transcribed-chat-reply"}
            if prior and prior["payload"] != value:
                raise ValueError("original_claim_response_changed")
            if not prior:
                self.put("response", payload["roundId"], value, connection=connection, expected_version=0)
        return value

    def close_round(self, payload: dict) -> dict:
        with self.state.transaction() as connection:
            row = self.state.get("governance_round", payload["roundId"], connection=connection)
            current = self.get("round", payload["roundId"], connection=connection)
            response = self.get("response", payload["roundId"], connection=connection)
            if response["replyDigest"] != payload["replyDigest"]:
                raise ValueError("claim_reply_changed")
            if current["status"] == "closed":
                return current
            claims = [r["payload"] for r in self.state.list("governance_claim", connection=connection)
                      if r["payload"]["roundId"] == payload["roundId"]]
            value = {**current, "status": "closed", "closedAt": time.time(),
                     "claimDigest": digest(claims), "replyDigest": response["replyDigest"]}
            self.put("round", payload["roundId"], value, connection=connection, expected_version=row["version"])
        return value

    def require_closed(self, payload: dict) -> dict:
        current = self.get("round", payload["roundId"])
        if current["status"] != "closed":
            raise ValueError("claim_round_awaiting_user_reply")
        if payload.get("inventoryDigest") != current["inventoryDigest"]:
            raise ValueError("operation_inventory_changed")
        return current

    def adopt_legacy_claim(self, payload: dict) -> dict:
        """Keep historical identity; transfer only after original-source reconciliation."""
        previous = self.state.get(payload["domain"], payload["key"])
        if payload["domain"] not in {"management_worktree", "path_claims", "patch_owner"} or not previous:
            raise ValueError("legacy_owner_missing")
        if digest(previous["payload"]) != payload["previousDigest"]:
            raise ValueError("legacy_owner_changed")
        evidence, legacy = payload["evidence"], previous["payload"]
        registration = self.get("repository", payload["repositoryId"])
        if legacy.get("repositoryId") != payload["repositoryId"]:
            raise ValueError("legacy_repository_identity_unproved")
        source = physical(legacy.get("worktreeRoot") or legacy.get("pathRoot") or registration["canonicalCheckout"])
        hashes = legacy.get("sourceHashes") or legacy.get("inputHashes")
        references = legacy.get("retainedRefs")
        jobs = legacy.get("nativeJobIds") or ([legacy["nativeJobId"]] if legacy.get("nativeJobId") else [])
        if not isinstance(hashes, dict) or not hashes or not references or not jobs:
            raise ValueError("historical_registration_is_not_current_ownership_proof")
        for relative, expected in hashes.items():
            from .inventory import authentication_path
            if authentication_path(Path(relative)):
                raise ValueError("legacy_authentication_content_requires_review_without_reading")
            current = file_fact(physical(source / relative_source(relative)))["sha256"]
            if current != expected:
                raise ValueError("legacy_source_input_changed")
        for item in references:
            if text_git(source, "rev-parse", "--verify", item["ref"]) != item["sha"]:
                raise ValueError("legacy_retained_reference_changed")
        job_proofs = []
        for key in jobs:
            row = self.state.get("native_job", key)
            job = row["payload"] if row else {}
            if (job.get("status") != "terminal" or job.get("completeProof", {}).get("complete") is not True
                    or job.get("repositoryId") != payload["repositoryId"]
                    or job.get("requestId") != legacy.get("requestId")
                    or job.get("attemptId") != legacy.get("attemptId")
                    or not legacy.get("requestId") or not legacy.get("attemptId")):
                raise ValueError("legacy_original_request_attempt_requires_reconciliation")
            job_proofs.append({"key": key, "digest": digest(job)})
        evidence = {"sourceHashes": hashes, "retainedRefs": references, "originalNativeJobs": job_proofs,
                    "previousDriverDigest": legacy.get("driverDigest")}
        value = {"domain": payload["domain"], "key": payload["key"], "previousDigest": payload["previousDigest"],
                 "previousVersion": previous["version"], "evidence": evidence, **self.runtime,
                 "status": "verified-adoption", "historicalRecordPreserved": True}
        self.put("legacy_adoption", digest(value), value)
        return value
