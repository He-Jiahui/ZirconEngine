from __future__ import annotations

import os
import tempfile
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator

from .models import CoordinatorError


WINDOWS_CHECKOUT_PATH_BUDGET = 248


class IsolatedPatchCheckoutRootAllocator:
    """Allocate a private validation root without exceeding Windows path limits."""

    def __init__(
        self,
        repo_root: str | Path,
        *,
        base_candidates: tuple[Path, ...] | None = None,
    ) -> None:
        self.repo_root = Path(repo_root).resolve()
        self.base_candidates = base_candidates

    @contextmanager
    def allocate(self, tracked_paths: tuple[str, ...]) -> Iterator[Path]:
        if os.name != "nt":
            with tempfile.TemporaryDirectory(
                prefix="zircon-isolated-patch-"
            ) as directory:
                yield Path(directory).resolve()
            return

        failures: list[dict[str, object]] = []
        for base in self._windows_bases():
            temporary: tempfile.TemporaryDirectory[str] | None = None
            try:
                temporary = tempfile.TemporaryDirectory(prefix=".zr-ip-", dir=base)
                root = Path(temporary.name).absolute()
                self.require_private_root(root)
                maximum = self.maximum_path_length(root, tracked_paths)
                if maximum > WINDOWS_CHECKOUT_PATH_BUDGET:
                    failures.append(
                        {
                            "base": str(base),
                            "maximumPathLength": maximum,
                            "reason": "path_budget_exceeded",
                        }
                    )
                    temporary.cleanup()
                    continue
            except (CoordinatorError, OSError) as error:
                if temporary is not None:
                    temporary.cleanup()
                failures.append({"base": str(base), "reason": str(error)})
                continue
            try:
                yield root
            finally:
                temporary.cleanup()
            return
        raise CoordinatorError(
            "isolated_patch_checkout_root_unavailable",
            "No private writable Windows checkout root satisfies the path budget",
            details={
                "pathBudget": WINDOWS_CHECKOUT_PATH_BUDGET,
                "attempts": failures,
            },
        )

    def _windows_bases(self) -> tuple[Path, ...]:
        candidates = self.base_candidates or (
            Path.home(),
            self.repo_root.parent,
            Path(tempfile.gettempdir()),
        )
        unique: list[Path] = []
        seen: set[str] = set()
        for candidate in candidates:
            absolute = Path(candidate).absolute()
            key = os.path.normcase(str(absolute))
            if key not in seen:
                seen.add(key)
                unique.append(absolute)
        return tuple(unique)

    @classmethod
    def maximum_path_length(cls, root: Path, tracked_paths: tuple[str, ...]) -> int:
        return max(
            (
                cls._windows_path_length(root / relative.replace("/", os.sep))
                for relative in tracked_paths
            ),
            default=cls._windows_path_length(root),
        )

    @staticmethod
    def _windows_path_length(path: Path) -> int:
        return len(str(path).encode("utf-16-le", errors="surrogatepass")) // 2

    @staticmethod
    def require_private_root(root: Path) -> None:
        is_junction = getattr(root, "is_junction", lambda: False)
        if not root.is_dir() or root.is_symlink() or is_junction():
            raise CoordinatorError(
                "isolated_patch_checkout_root_unsafe",
                "Isolated validation root must be a private non-reparse directory",
                details={"root": str(root)},
            )
