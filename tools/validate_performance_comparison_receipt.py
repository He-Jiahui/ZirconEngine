"""Validate signed, comparable performance evidence before a baseline promotion."""

from __future__ import annotations

import argparse
import base64
import binascii
import copy
import hashlib
import json
import math
import random
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

if __package__ in {None, ""}:
    # Support the repository's direct ``python tools/<script>.py`` invocation.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

from tools.validate_render_measurement_evidence import validate_render_measurement_evidence


PERFORMANCE_COMPARISON_RECEIPT_SCHEMA = "zircon_performance_comparison_receipt_v1"
_SIGNATURE_ALGORITHM = "ed25519"
_SHA256_HEX_LENGTH = 64
_BOOTSTRAP_RESAMPLE_COUNT = 2_000
_METRIC_UNITS = {
    "cpu_mesh_encode_ns": "ns",
    "gpu_frame_ns": "ns",
    "board_power_w": "watts",
}
_ARTIFACT_FIELDS = (
    "png_path",
    "rdc_cold_path",
    "rdc_warm_path",
    "graph_dump_path",
)


@dataclass(frozen=True)
class PerformanceComparisonReceipt:
    metric: str
    baseline_median: float
    candidate_median: float
    median_ratio: float
    baseline_p95: float
    candidate_p95: float
    p95_ratio: float
    allows_baseline_update: bool


def sign_performance_comparison_receipt(
    receipt: Mapping[str, Any], private_key: Ed25519PrivateKey
) -> dict[str, Any]:
    """Return a copy signed over every receipt field except ``signature``.

    This helper is intentionally small: production signers own their private key,
    while the validator only accepts a separately supplied trusted public-key map.
    """

    if not isinstance(private_key, Ed25519PrivateKey):
        raise TypeError("private_key must be an Ed25519PrivateKey")
    document = _copy_json_object(receipt, "receipt")
    if "signature" in document:
        raise RuntimeError("Performance comparison receipt must not be pre-signed")
    signature = private_key.sign(_canonical_payload(document))
    document["signature"] = {
        "algorithm": _SIGNATURE_ALGORITHM,
        "value": base64.b64encode(signature).decode("ascii"),
    }
    return document


def validate_performance_comparison_receipt(
    receipt: Mapping[str, Any] | str | Path,
    *,
    baseline_report: str | Path,
    candidate_report: str | Path,
    baseline_artifact_root: str | Path,
    candidate_artifact_root: str | Path,
    trusted_acceptor_public_keys: Mapping[str, bytes | Ed25519PublicKey],
) -> PerformanceComparisonReceipt:
    """Verify a signed comparison receipt and its two immutable input reports.

    This validates the reproducibility of a signed comparison bundle. It cannot
    authorize a baseline promotion: trusted promotion policy, worker capture
    receipts, and candidate binding are owned by the Coordinator contract. A
    receipt that self-asserts ``baseline_update`` is rejected until that contract
    is available.
    """

    document = _load_receipt(receipt)
    _require_exact_keys(
        document,
        {
            "schema",
            "scenario",
            "baseline",
            "candidate",
            "environment",
            "comparison",
            "acceptance",
            "signature",
        },
        "receipt",
    )
    _require_exact_value(
        document["schema"], PERFORMANCE_COMPARISON_RECEIPT_SCHEMA, "receipt.schema"
    )
    acceptor_id = _verify_signature(document, trusted_acceptor_public_keys)

    baseline_path = Path(baseline_report)
    candidate_path = Path(candidate_report)
    baseline_document = _load_json_object(baseline_path, "baseline report")
    candidate_document = _load_json_object(candidate_path, "candidate report")
    _validate_distinct_reports(baseline_path, candidate_path)
    _validate_distinct_capture_tickets(baseline_document, candidate_document)

    # Existing sidecar validation proves the signed raw-artifact paths are present
    # and contained before their bytes are bound into this comparison receipt.
    validate_render_measurement_evidence(
        baseline_path,
        require_artifacts=True,
        artifact_root=baseline_artifact_root,
    )
    validate_render_measurement_evidence(
        candidate_path,
        require_artifacts=True,
        artifact_root=candidate_artifact_root,
    )
    baseline = _validate_report_binding(
        document["baseline"],
        baseline_path,
        baseline_document,
        baseline_artifact_root,
        "baseline",
    )
    candidate = _validate_report_binding(
        document["candidate"],
        candidate_path,
        candidate_document,
        candidate_artifact_root,
        "candidate",
    )

    _validate_scenario(document["scenario"], baseline_document, candidate_document)
    _validate_environment(document["environment"], baseline_document, candidate_document)
    comparison = _validate_comparison(
        document["comparison"], baseline_document, candidate_document
    )
    allows_baseline_update = _validate_acceptance(
        document["acceptance"],
        acceptor_id,
        baseline_document,
        candidate_document,
        comparison,
    )
    if comparison["decision"] == "accepted":
        _validate_budget(comparison)
    return PerformanceComparisonReceipt(
        metric=comparison["metric"],
        baseline_median=comparison["effect"]["baseline_median"],
        candidate_median=comparison["effect"]["candidate_median"],
        median_ratio=comparison["effect"]["median_ratio"],
        baseline_p95=comparison["effect"]["baseline_p95"],
        candidate_p95=comparison["effect"]["candidate_p95"],
        p95_ratio=comparison["effect"]["p95_ratio"],
        allows_baseline_update=allows_baseline_update,
    )


def _load_receipt(receipt: Mapping[str, Any] | str | Path) -> dict[str, Any]:
    if isinstance(receipt, (str, Path)):
        return _load_json_object(Path(receipt), "receipt")
    return _copy_json_object(receipt, "receipt")


def _copy_json_object(value: Mapping[str, Any], label: str) -> dict[str, Any]:
    if not isinstance(value, Mapping):
        raise RuntimeError(f"Performance comparison {label} must be a JSON object")
    try:
        copied = copy.deepcopy(dict(value))
        json.dumps(copied, allow_nan=False)
    except (TypeError, ValueError) as error:
        raise RuntimeError(f"Performance comparison {label} must be JSON-serializable") from error
    return copied


def _load_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(f"Performance comparison {label} is unavailable: {path}") from error
    except json.JSONDecodeError as error:
        raise RuntimeError(f"Performance comparison {label} is not valid JSON: {path}") from error
    if not isinstance(document, dict):
        raise RuntimeError(f"Performance comparison {label} must be a JSON object: {path}")
    return document


def _verify_signature(
    document: Mapping[str, Any], trusted_acceptors: Mapping[str, bytes | Ed25519PublicKey]
) -> str:
    signature = _require_object(document["signature"], "receipt.signature")
    _require_exact_keys(signature, {"algorithm", "value"}, "receipt.signature")
    _require_exact_value(signature["algorithm"], _SIGNATURE_ALGORITHM, "receipt.signature.algorithm")
    signature_value = _require_non_empty_string(signature["value"], "receipt.signature.value")
    try:
        raw_signature = base64.b64decode(signature_value, validate=True)
    except (ValueError, binascii.Error) as error:
        raise RuntimeError("Performance comparison receipt signature is not base64") from error
    if len(raw_signature) != 64:
        raise RuntimeError("Performance comparison receipt signature has an invalid length")

    acceptance = _require_object(document["acceptance"], "receipt.acceptance")
    acceptor_id = _require_non_empty_string(acceptance.get("acceptor_id"), "acceptance.acceptor_id")
    public_key = trusted_acceptors.get(acceptor_id)
    if public_key is None:
        raise RuntimeError(
            f"Performance comparison receipt acceptor is not trusted: acceptor_id={acceptor_id}"
        )
    verifier = _coerce_public_key(public_key, acceptor_id)
    unsigned = dict(document)
    unsigned.pop("signature", None)
    try:
        verifier.verify(raw_signature, _canonical_payload(unsigned))
    except InvalidSignature as error:
        raise RuntimeError("Performance comparison receipt signature is invalid") from error
    return acceptor_id


def _coerce_public_key(value: bytes | Ed25519PublicKey, acceptor_id: str) -> Ed25519PublicKey:
    if isinstance(value, Ed25519PublicKey):
        return value
    if not isinstance(value, bytes) or len(value) != 32:
        raise RuntimeError(
            "Performance comparison trusted acceptor key must be an Ed25519 raw public key: "
            f"acceptor_id={acceptor_id}"
        )
    try:
        return Ed25519PublicKey.from_public_bytes(value)
    except ValueError as error:
        raise RuntimeError(
            f"Performance comparison trusted acceptor key is invalid: acceptor_id={acceptor_id}"
        ) from error


def _validate_report_binding(
    value: Any,
    report_path: Path,
    report: Mapping[str, Any],
    artifact_root: str | Path,
    label: str,
) -> Mapping[str, Any]:
    binding = _require_object(value, f"receipt.{label}")
    _require_exact_keys(
        binding,
        {"report_sha256", "artifact_sha256", "build_set"},
        f"receipt.{label}",
    )
    expected_hash = _require_sha256(binding["report_sha256"], f"{label}.report_sha256")
    actual_hash = hashlib.sha256(report_path.read_bytes()).hexdigest()
    if actual_hash != expected_hash:
        raise RuntimeError(
            f"Performance comparison {label}.report_sha256 does not match its report: {report_path}"
        )
    _validate_artifact_hashes(
        binding["artifact_sha256"], report, artifact_root, label
    )
    source = _require_object(report.get("source"), f"{label} report.source")
    build_set = _require_object(binding["build_set"], f"{label}.build_set")
    _require_exact_keys(
        build_set,
        {
            "revision",
            "source_fingerprint",
            "target_profile",
            "toolchain",
            "binary_sha256",
            "symbol_sha256",
        },
        f"{label}.build_set",
    )
    for field in ("revision", "source_fingerprint", "target_profile", "toolchain"):
        _require_non_empty_string(build_set[field], f"{label}.build_set.{field}")
    _require_revision(build_set["revision"], f"{label}.build_set.revision")
    _require_sha256(build_set["source_fingerprint"], f"{label}.build_set.source_fingerprint")
    for field in ("binary_sha256", "symbol_sha256"):
        _require_sha256(build_set[field], f"{label}.build_set.{field}")
    if build_set["target_profile"] not in {"release", "profiling"}:
        raise RuntimeError(
            f"Performance comparison {label}.build_set.target_profile is not optimized"
        )
    for field in ("revision", "source_fingerprint"):
        if build_set[field] != source.get(field):
            raise RuntimeError(
                f"Performance comparison {label}.build_set.{field} does not match the report source"
            )
    return binding


def _validate_distinct_reports(baseline_path: Path, candidate_path: Path) -> None:
    baseline_hash = hashlib.sha256(baseline_path.read_bytes()).digest()
    candidate_hash = hashlib.sha256(candidate_path.read_bytes()).digest()
    if baseline_hash == candidate_hash:
        raise RuntimeError(
            "Performance comparison baseline and candidate reports must be distinct"
        )


def _validate_distinct_capture_tickets(
    baseline: Mapping[str, Any], candidate: Mapping[str, Any]
) -> None:
    baseline_source = _require_object(baseline.get("source"), "baseline report.source")
    candidate_source = _require_object(candidate.get("source"), "candidate report.source")
    baseline_ticket = _require_non_empty_string(
        baseline_source.get("validation_ticket"), "baseline source.validation_ticket"
    )
    candidate_ticket = _require_non_empty_string(
        candidate_source.get("validation_ticket"), "candidate source.validation_ticket"
    )
    if baseline_ticket == candidate_ticket:
        raise RuntimeError(
            "Performance comparison capture validation tickets must be distinct"
        )


def _validate_artifact_hashes(
    value: Any,
    report: Mapping[str, Any],
    artifact_root: str | Path,
    label: str,
) -> None:
    hashes = _require_object(value, f"{label}.artifact_sha256")
    _require_exact_keys(
        hashes, set(_ARTIFACT_FIELDS), f"{label}.artifact_sha256"
    )
    artifacts = _require_object(report.get("artifacts"), f"{label} report.artifacts")
    root = Path(artifact_root).resolve(strict=True)
    for field in _ARTIFACT_FIELDS:
        relative_path = _require_non_empty_string(
            artifacts.get(field), f"{label} report.artifacts.{field}"
        )
        artifact_path = (root / Path(relative_path)).resolve(strict=True)
        if root not in artifact_path.parents:
            raise RuntimeError(
                f"Performance comparison {label} artifact escapes its root: {artifact_path}"
            )
        expected_hash = _require_sha256(
            hashes[field], f"{label}.artifact_sha256.{field}"
        )
        actual_hash = hashlib.sha256(artifact_path.read_bytes()).hexdigest()
        if actual_hash != expected_hash:
            raise RuntimeError(
                "Performance comparison "
                f"{label}.artifact_sha256.{field} does not match report artifact: "
                f"{artifact_path}"
            )


def _validate_scenario(
    value: Any, baseline: Mapping[str, Any], candidate: Mapping[str, Any]
) -> None:
    scenario = _require_object(value, "receipt.scenario")
    _require_exact_keys(
        scenario, {"id", "input_fingerprint", "workload_fingerprint"}, "receipt.scenario"
    )
    scenario_id = _require_non_empty_string(scenario["id"], "scenario.id")
    input_fingerprint = _require_sha256(
        scenario["input_fingerprint"], "scenario.input_fingerprint"
    )
    workload_fingerprint = _require_sha256(
        scenario["workload_fingerprint"], "scenario.workload_fingerprint"
    )
    baseline_workload = _require_object(baseline.get("workload"), "baseline report.workload")
    candidate_workload = _require_object(candidate.get("workload"), "candidate report.workload")
    if baseline_workload != candidate_workload:
        raise RuntimeError("Performance comparison reports use different workloads")
    if scenario_id != _scenario_id(baseline_workload):
        raise RuntimeError("Performance comparison scenario ID is not bound to reports")
    if input_fingerprint != _scenario_input_fingerprint(baseline_workload):
        raise RuntimeError(
            "Performance comparison scenario input fingerprint is not bound to reports"
        )
    if _sha256_json(baseline_workload) != workload_fingerprint:
        raise RuntimeError("Performance comparison scenario workload fingerprint is not bound to reports")


def _scenario_input_fingerprint(workload: Mapping[str, Any]) -> str:
    """Bind the scenario's stable render inputs independently from sampling cadence."""

    return _sha256_json(
        {
            field: workload[field]
            for field in (
                "name",
                "variant",
                "resolution",
                "quality_profile",
                "camera_fingerprint",
            )
        }
    )


def _scenario_id(workload: Mapping[str, Any]) -> str:
    return "render19.{}.{}".format(workload["name"], workload["variant"])


def _validate_environment(
    value: Any, baseline: Mapping[str, Any], candidate: Mapping[str, Any]
) -> None:
    environment = _require_object(value, "receipt.environment")
    _require_exact_keys(
        environment,
        {
            "hardware_fingerprint",
            "adapter_fingerprint",
            "os_version",
            "driver_version",
            "power_policy",
        },
        "receipt.environment",
    )
    _require_sha256(environment["hardware_fingerprint"], "environment.hardware_fingerprint")
    adapter_fingerprint = _require_sha256(
        environment["adapter_fingerprint"], "environment.adapter_fingerprint"
    )
    for field in ("os_version", "driver_version", "power_policy"):
        _require_non_empty_string(environment[field], f"environment.{field}")
    baseline_adapter = _require_object(baseline.get("adapter"), "baseline report.adapter")
    candidate_adapter = _require_object(candidate.get("adapter"), "candidate report.adapter")
    if baseline_adapter != candidate_adapter:
        raise RuntimeError("Performance comparison reports use different adapters")
    if _sha256_json(baseline_adapter) != adapter_fingerprint:
        raise RuntimeError("Performance comparison adapter fingerprint is not bound to reports")
    if baseline_adapter.get("driver") != environment["driver_version"]:
        raise RuntimeError("Performance comparison environment driver does not match reports")


def _validate_comparison(
    value: Any, baseline: Mapping[str, Any], candidate: Mapping[str, Any]
) -> dict[str, Any]:
    comparison = _copy_json_object(_require_object(value, "receipt.comparison"), "receipt.comparison")
    _require_exact_keys(
        comparison,
        {
            "metric",
            "unit",
            "direction",
            "method",
            "baseline_samples",
            "candidate_samples",
            "effect",
            "confidence",
            "budget",
            "decision",
        },
        "receipt.comparison",
    )
    metric = comparison["metric"]
    if metric not in _METRIC_UNITS:
        raise RuntimeError(f"Performance comparison metric is unsupported: {metric!r}")
    _require_exact_value(comparison["unit"], _METRIC_UNITS[metric], "comparison.unit")
    _require_exact_value(comparison["direction"], "lower_is_better", "comparison.direction")
    _require_exact_value(comparison["method"], "raw-sample-distribution-v1", "comparison.method")
    if comparison["decision"] not in {"accepted", "rejected"}:
        raise RuntimeError("Performance comparison comparison.decision must be accepted or rejected")

    baseline_samples = _validate_samples(comparison["baseline_samples"], "comparison.baseline_samples")
    candidate_samples = _validate_samples(comparison["candidate_samples"], "comparison.candidate_samples")
    baseline_statistics = _validate_samples_against_report(
        baseline_samples, baseline, metric, "baseline"
    )
    candidate_statistics = _validate_samples_against_report(
        candidate_samples, candidate, metric, "candidate"
    )
    baseline_median = baseline_statistics["median"]
    candidate_median = candidate_statistics["median"]
    median_ratio = (candidate_median - baseline_median) / baseline_median
    p95_ratio = (
        candidate_statistics["p95"] - baseline_statistics["p95"]
    ) / baseline_statistics["p95"]
    effect = _validate_effect(
        comparison["effect"],
        baseline_statistics,
        candidate_statistics,
        median_ratio,
        p95_ratio,
    )
    _validate_confidence(
        comparison["confidence"], median_ratio, baseline_samples, candidate_samples
    )
    _validate_budget_shape(comparison["budget"])
    comparison["effect"] = effect
    return comparison


def _validate_samples(value: Any, label: str) -> list[float]:
    if not isinstance(value, list) or not value:
        raise RuntimeError(f"Performance comparison {label} must be a non-empty array")
    samples: list[float] = []
    for index, sample in enumerate(value):
        number = _require_finite_number(sample, f"{label}[{index}]")
        if number <= 0:
            raise RuntimeError(f"Performance comparison {label}[{index}] must be positive")
        samples.append(number)
    return samples


def _validate_samples_against_report(
    samples: list[float], report: Mapping[str, Any], metric: str, label: str
) -> dict[str, float]:
    observations = _require_object(report.get("observations"), f"{label} report.observations")
    valid_frame_count = observations.get("valid_frame_count")
    if isinstance(valid_frame_count, bool) or not isinstance(valid_frame_count, int):
        raise RuntimeError(f"Performance comparison {label} report valid_frame_count is invalid")
    if len(samples) != valid_frame_count:
        raise RuntimeError(
            f"Performance comparison {label} samples do not match report valid_frame_count"
        )
    report_statistics = _require_object(observations.get(metric), f"{label} report {metric}")
    actual_statistics = _statistics(samples)
    for field, expected in actual_statistics.items():
        reported = _require_finite_number(report_statistics.get(field), f"{label} report {metric}.{field}")
        if not math.isclose(reported, expected, rel_tol=0.0, abs_tol=1e-9):
            raise RuntimeError(
                f"Performance comparison {label} samples do not match report {metric}.{field}"
            )
    return actual_statistics


def _statistics(samples: list[float]) -> dict[str, float]:
    ordered = sorted(samples)
    count = len(ordered)
    midpoint = count // 2
    if count % 2:
        median = ordered[midpoint]
    else:
        median = (ordered[midpoint - 1] + ordered[midpoint]) / 2.0
    p95 = ordered[math.ceil(count * 0.95) - 1]
    deviations = sorted(abs(sample - median) for sample in ordered)
    if count % 2:
        mad = deviations[midpoint]
    else:
        mad = (deviations[midpoint - 1] + deviations[midpoint]) / 2.0
    return {"median": median, "p95": p95, "mad": mad}


def _validate_effect(
    value: Any,
    baseline: Mapping[str, float],
    candidate: Mapping[str, float],
    median_ratio: float,
    p95_ratio: float,
) -> dict[str, float]:
    effect = _require_object(value, "comparison.effect")
    _require_exact_keys(
        effect,
        {
            "baseline_median",
            "candidate_median",
            "median_ratio",
            "baseline_p95",
            "candidate_p95",
            "p95_ratio",
            "baseline_mad",
            "candidate_mad",
        },
        "comparison.effect",
    )
    expected = {
        "baseline_median": baseline["median"],
        "candidate_median": candidate["median"],
        "median_ratio": median_ratio,
        "baseline_p95": baseline["p95"],
        "candidate_p95": candidate["p95"],
        "p95_ratio": p95_ratio,
        "baseline_mad": baseline["mad"],
        "candidate_mad": candidate["mad"],
    }
    normalized: dict[str, float] = {}
    for field, expected_value in expected.items():
        actual = _require_finite_number(effect[field], f"comparison.effect.{field}")
        if not math.isclose(actual, expected_value, rel_tol=0.0, abs_tol=1e-9):
            raise RuntimeError(f"Performance comparison comparison.effect.{field} is not reproducible")
        normalized[field] = actual
    return normalized


def _validate_confidence(
    value: Any,
    median_ratio: float,
    baseline_samples: list[float],
    candidate_samples: list[float],
) -> None:
    confidence = _require_object(value, "comparison.confidence")
    _require_exact_keys(
        confidence, {"method", "level", "lower_ratio", "upper_ratio"}, "comparison.confidence"
    )
    _require_exact_value(confidence["method"], "bootstrap-percentile-v1", "comparison.confidence.method")
    level = _require_finite_number(confidence["level"], "comparison.confidence.level")
    lower = _require_finite_number(confidence["lower_ratio"], "comparison.confidence.lower_ratio")
    upper = _require_finite_number(confidence["upper_ratio"], "comparison.confidence.upper_ratio")
    if not 0.0 < level < 1.0 or lower > median_ratio or median_ratio > upper:
        raise RuntimeError("Performance comparison confidence interval does not contain the median ratio")
    bootstrap_lower, bootstrap_upper = _bootstrap_ratio_interval(
        baseline_samples, candidate_samples, level
    )
    if lower > bootstrap_lower or upper < bootstrap_upper:
        raise RuntimeError(
            "Performance comparison confidence interval does not contain bootstrap interval"
        )


def _bootstrap_ratio_interval(
    baseline_samples: list[float],
    candidate_samples: list[float],
    level: float,
    *,
    statistic: str = "median",
) -> tuple[float, float]:
    if statistic not in {"median", "p95"}:
        raise ValueError(f"Unsupported bootstrap statistic: {statistic!r}")
    seed_document = {
        "baseline_samples": baseline_samples,
        "candidate_samples": candidate_samples,
        "statistic": statistic,
    }
    seed = int.from_bytes(hashlib.sha256(_canonical_payload(seed_document)).digest(), "big")
    random_generator = random.Random(seed)
    baseline_groups, baseline_values = _ordered_sample_groups(baseline_samples)
    candidate_groups, candidate_values = _ordered_sample_groups(candidate_samples)
    ratios = []
    for _ in range(_BOOTSTRAP_RESAMPLE_COUNT):
        baseline_value = _resampled_statistic(
            baseline_samples, baseline_groups, baseline_values, random_generator, statistic
        )
        candidate_value = _resampled_statistic(
            candidate_samples, candidate_groups, candidate_values, random_generator, statistic
        )
        ratios.append((candidate_value - baseline_value) / baseline_value)
    tail_probability = (1.0 - level) / 2.0
    return (
        _percentile(ratios, tail_probability),
        _percentile(ratios, 1.0 - tail_probability),
    )


def _resampled_statistic(
    samples: list[float],
    sample_groups: list[int],
    ordered_values: list[float],
    random_generator: random.Random,
    statistic: str,
) -> float:
    count = len(samples)
    sample_counts = [0] * len(ordered_values)
    for _ in range(count):
        sample_counts[sample_groups[random_generator.randrange(count)]] += 1
    if statistic == "median":
        midpoint = count // 2
        ranks = (midpoint,) if count % 2 else (midpoint - 1, midpoint)
    else:
        ranks = (math.ceil(count * 0.95) - 1,)

    values: list[float] = []
    cumulative_count = 0
    rank_index = 0
    for value, value_count in zip(ordered_values, sample_counts, strict=True):
        cumulative_count += value_count
        while rank_index < len(ranks) and cumulative_count > ranks[rank_index]:
            values.append(value)
            rank_index += 1
        if rank_index == len(ranks):
            break
    return values[0] if len(values) == 1 else (values[0] + values[1]) / 2.0


def _ordered_sample_groups(samples: list[float]) -> tuple[list[int], list[float]]:
    ordered_values = sorted(set(samples))
    groups_by_value = {value: index for index, value in enumerate(ordered_values)}
    return [groups_by_value[value] for value in samples], ordered_values


def _percentile(samples: list[float], probability: float) -> float:
    ordered = sorted(samples)
    position = probability * (len(ordered) - 1)
    lower_index = math.floor(position)
    upper_index = math.ceil(position)
    lower_value = ordered[lower_index]
    upper_value = ordered[upper_index]
    return lower_value + (upper_value - lower_value) * (position - lower_index)


def _validate_budget_shape(value: Any) -> None:
    budget = _require_object(value, "comparison.budget")
    _require_exact_keys(
        budget,
        {
            "max_regression_ratio",
            "max_regression_absolute",
            "max_p95_regression_ratio",
        },
        "comparison.budget",
    )
    for field in budget:
        if _require_finite_number(budget[field], f"comparison.budget.{field}") < 0:
            raise RuntimeError(f"Performance comparison comparison.budget.{field} must be non-negative")


def _validate_acceptance(
    value: Any,
    acceptor_id: str,
    baseline: Mapping[str, Any],
    candidate: Mapping[str, Any],
    comparison: Mapping[str, Any],
) -> bool:
    acceptance = _require_object(value, "receipt.acceptance")
    _require_exact_keys(
        acceptance, {"acceptor_id", "accepted_at", "baseline_update"}, "receipt.acceptance"
    )
    if acceptance["acceptor_id"] != acceptor_id:
        raise RuntimeError("Performance comparison signature and acceptance acceptor disagree")
    _require_non_empty_string(acceptance["accepted_at"], "acceptance.accepted_at")
    if not isinstance(acceptance["baseline_update"], bool):
        raise RuntimeError("Performance comparison acceptance.baseline_update must be a boolean")
    baseline_source = _require_object(baseline.get("source"), "baseline report.source")
    candidate_source = _require_object(candidate.get("source"), "candidate report.source")
    capture_sessions = {
        _require_non_empty_string(baseline_source.get("session_id"), "baseline source.session_id"),
        _require_non_empty_string(candidate_source.get("session_id"), "candidate source.session_id"),
    }
    if acceptor_id in capture_sessions:
        raise RuntimeError(
            "Performance comparison acceptor must be independent of both capture sessions"
        )
    if acceptance["baseline_update"] and comparison["decision"] != "accepted":
        raise RuntimeError("Performance comparison rejected receipt cannot update a baseline")
    if acceptance["baseline_update"]:
        raise RuntimeError(
            "Performance comparison baseline updates require a Coordinator promotion contract"
        )
    return False


def _validate_budget(comparison: Mapping[str, Any]) -> None:
    effect = comparison["effect"]
    budget = comparison["budget"]
    absolute_regression = effect["candidate_median"] - effect["baseline_median"]
    if (
        effect["median_ratio"] > budget["max_regression_ratio"]
        or absolute_regression > budget["max_regression_absolute"]
        or effect["p95_ratio"] > budget["max_p95_regression_ratio"]
    ):
        raise RuntimeError("Performance comparison candidate exceeds the regression budget")
    confidence = _require_object(comparison["confidence"], "comparison.confidence")
    median_upper_ratio = _require_finite_number(
        confidence["upper_ratio"], "comparison.confidence.upper_ratio"
    )
    if median_upper_ratio > budget["max_regression_ratio"]:
        raise RuntimeError(
            "Performance comparison median confidence upper bound exceeds the regression budget"
        )
    baseline_samples = _validate_samples(
        comparison["baseline_samples"], "comparison.baseline_samples"
    )
    candidate_samples = _validate_samples(
        comparison["candidate_samples"], "comparison.candidate_samples"
    )
    _, p95_upper_ratio = _bootstrap_ratio_interval(
        baseline_samples, candidate_samples, confidence["level"], statistic="p95"
    )
    if p95_upper_ratio > budget["max_p95_regression_ratio"]:
        raise RuntimeError(
            "Performance comparison P95 confidence upper bound exceeds the regression budget"
        )


def _canonical_payload(value: Mapping[str, Any]) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def _sha256_json(value: Mapping[str, Any]) -> str:
    return hashlib.sha256(_canonical_payload(value)).hexdigest()


def _require_object(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise RuntimeError(f"Performance comparison {label} must be an object")
    return value


def _require_exact_keys(value: Mapping[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    missing = sorted(expected - actual)
    unexpected = sorted(actual - expected)
    if missing or unexpected:
        details = []
        if missing:
            details.append(f"missing={','.join(missing)}")
        if unexpected:
            details.append(f"unexpected={','.join(unexpected)}")
        raise RuntimeError(
            f"Performance comparison {label} fields do not match the contract: {' '.join(details)}"
        )


def _require_exact_value(value: Any, expected: str, label: str) -> None:
    if value != expected:
        raise RuntimeError(f"Performance comparison {label} must equal {expected!r}")


def _require_non_empty_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise RuntimeError(f"Performance comparison {label} must be a non-empty string")
    return value


def _require_sha256(value: Any, label: str) -> str:
    normalized = _require_non_empty_string(value, label).casefold()
    if len(normalized) != _SHA256_HEX_LENGTH or any(
        character not in "0123456789abcdef" for character in normalized
    ):
        raise RuntimeError(f"Performance comparison {label} must be a lowercase SHA-256")
    return normalized


def _require_revision(value: Any, label: str) -> str:
    normalized = _require_non_empty_string(value, label).casefold()
    if len(normalized) not in {40, 64} or any(
        character not in "0123456789abcdef" for character in normalized
    ):
        raise RuntimeError(f"Performance comparison {label} must be a 40- or 64-hex revision")
    return normalized


def _require_finite_number(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise RuntimeError(f"Performance comparison {label} must be a finite number")
    return float(value)


def _parse_public_keys(values: list[str]) -> dict[str, bytes]:
    keys: dict[str, bytes] = {}
    for value in values:
        acceptor_id, separator, encoded_key = value.partition("=")
        if not separator or not acceptor_id or acceptor_id in keys:
            raise ValueError("--acceptor-public-key requires unique ACCEPTOR_ID=BASE64_KEY values")
        try:
            key = base64.b64decode(encoded_key, validate=True)
        except (ValueError, binascii.Error) as error:
            raise ValueError("--acceptor-public-key must contain a base64 public key") from error
        if len(key) != 32:
            raise ValueError("--acceptor-public-key must contain a 32-byte Ed25519 public key")
        keys[acceptor_id] = key
    return keys


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate an independently signed Zircon performance comparison receipt."
    )
    parser.add_argument("receipt", type=Path, help="signed comparison receipt JSON")
    parser.add_argument("baseline_report", type=Path, help="baseline measured sidecar JSON")
    parser.add_argument("candidate_report", type=Path, help="candidate measured sidecar JSON")
    parser.add_argument(
        "--baseline-artifact-root",
        required=True,
        type=Path,
        help="root containing baseline receipt raw artifacts",
    )
    parser.add_argument(
        "--candidate-artifact-root",
        required=True,
        type=Path,
        help="root containing candidate receipt raw artifacts",
    )
    parser.add_argument(
        "--acceptor-public-key",
        action="append",
        default=[],
        metavar="ACCEPTOR_ID=BASE64_KEY",
        help="trusted raw Ed25519 public key; repeat for multiple acceptors",
    )
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        result = validate_performance_comparison_receipt(
            arguments.receipt,
            baseline_report=arguments.baseline_report,
            candidate_report=arguments.candidate_report,
            baseline_artifact_root=arguments.baseline_artifact_root,
            candidate_artifact_root=arguments.candidate_artifact_root,
            trusted_acceptor_public_keys=_parse_public_keys(arguments.acceptor_public_key),
        )
    except (OSError, RuntimeError, ValueError) as error:
        print(f"Performance comparison validation failed: {error}", file=sys.stderr)
        return 1
    print(
        json.dumps(
            {
                "allows_baseline_update": result.allows_baseline_update,
                "baseline_median": result.baseline_median,
                "baseline_p95": result.baseline_p95,
                "candidate_median": result.candidate_median,
                "candidate_p95": result.candidate_p95,
                "median_ratio": result.median_ratio,
                "metric": result.metric,
                "p95_ratio": result.p95_ratio,
                "schema": PERFORMANCE_COMPARISON_RECEIPT_SCHEMA,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
