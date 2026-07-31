from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .resource_budget import BurstDecision


BURST_TARGET_ROOT = Path("E:/cargo-targets/zircon-engine/burst")


@dataclass(frozen=True)
class CpuBurstRequest:
    reservation_id: str
    lane_scope: str
    burst_eligible: bool
    command: tuple[str, ...]
    target_dir: str | None


@dataclass(frozen=True)
class CpuBurstSelection:
    mode: str
    target_dir: Path | None
    reason: str

    def as_tuple(self) -> tuple[str, Path | None, str]:
        return self.mode, self.target_dir, self.reason


def select_cpu_burst(
    request: CpuBurstRequest,
    decision: BurstDecision,
    *,
    target_root: Path = BURST_TARGET_ROOT,
) -> CpuBurstSelection:
    """Choose an isolated check target only after bounded resource admission."""

    if not decision.allowed:
        return CpuBurstSelection("warm", None, decision.reason)
    if not is_burst_eligible_cpu_check(
        lane_scope=request.lane_scope,
        burst_eligible=request.burst_eligible,
        command=request.command,
        target_dir=request.target_dir,
    ):
        return CpuBurstSelection("warm", None, "not_eligible")
    return CpuBurstSelection(
        "burst",
        target_root / request.reservation_id,
        "allowed",
    )


def is_burst_eligible_cpu_check(
    *,
    lane_scope: str,
    burst_eligible: bool,
    command: tuple[str, ...],
    target_dir: str | None,
) -> bool:
    """Keep automatic isolated validation narrow before any resource probe runs."""

    is_check = command[:2] == ("cargo", "check")
    has_package = any(
        part in {"-p", "--package"} or part.startswith("--package=") for part in command
    )
    is_targeted_library_test = (
        command[:2] == ("cargo", "test")
        and "--lib" in command
        and has_package
        and "--workspace" not in command
    )

    return (
        lane_scope == "cpu"
        and burst_eligible
        and target_dir is None
        and (is_check or is_targeted_library_test)
    )
