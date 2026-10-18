"""Fail before scheduling persistent jobs when no matching runner is online."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


REQUIRED_LABELS = frozenset({"self-hosted", "windows", "x64"})


def fetch_runners(repository: str, token: str) -> list[dict]:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*/[A-Za-z0-9][A-Za-z0-9_.-]*", repository):
        raise ValueError("Expected an owner/repository GitHub identity.")
    if not token.strip():
        raise ValueError(
            "RUNNER_STATUS_TOKEN is missing. Configure a fine-grained repository "
            "token with Administration:read; GITHUB_TOKEN cannot list runners."
        )
    runners = []
    for page in range(1, 101):
        request = Request(
            f"https://api.github.com/repos/{repository}/actions/runners?per_page=100&page={page}",
            headers={
                "Authorization": f"Bearer {token}",
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
            },
        )
        with urlopen(request, timeout=15) as response:
            payload = json.load(response)
        batch = payload.get("runners") if isinstance(payload, dict) else None
        if not isinstance(batch, list) or not all(isinstance(row, dict) for row in batch):
            raise ValueError("GitHub returned an invalid runner inventory.")
        runners.extend(batch)
        if len(batch) < 100:
            return runners
    raise ValueError("Runner inventory exceeded the bounded pagination limit.")


def online_matching_runners(runners: list[dict]) -> list[dict]:
    matching = []
    for runner in runners:
        labels = runner.get("labels")
        if not isinstance(labels, list) or not all(
            isinstance(label, dict) and isinstance(label.get("name"), str)
            for label in labels
        ):
            raise ValueError("GitHub returned invalid runner labels.")
        names = {label["name"].casefold() for label in labels}
        if REQUIRED_LABELS <= names and runner.get("status") == "online":
            matching.append(runner)
    return matching


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    args = parser.parse_args()
    try:
        runners = fetch_runners(args.repository, os.environ.get("RUNNER_STATUS_TOKEN", ""))
        online = online_matching_runners(runners)
        if not online:
            raise ValueError(
                "No online self-hosted Windows X64 runner; persistent jobs will be skipped. "
                "Restore the managed runner before retrying this run."
            )
    except HTTPError as error:
        print(
            f"::error::Runner inventory HTTP {error.code}; verify RUNNER_STATUS_TOKEN "
            "has Administration:read for this repository.", file=sys.stderr,
        )
        return 1
    except (ValueError, URLError, TimeoutError) as error:
        print(f"::error::{error}", file=sys.stderr)
        return 1
    print(f"Runner availability: {len(online)} online self-hosted Windows X64 runner(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
