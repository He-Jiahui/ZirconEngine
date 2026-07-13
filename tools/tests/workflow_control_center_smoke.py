from __future__ import annotations

import argparse
from contextlib import closing
import http.cookiejar
import json
import re
import shutil
import sqlite3
import tempfile
import urllib.error
import urllib.request
from pathlib import Path

from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.tests.helpers import init_repo


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--read-only-console", action="store_true")
    parser.add_argument("--controlled-actions", action="store_true")
    args = parser.parse_args()
    if args.read_only_console == args.controlled_actions:
        parser.error("select exactly one smoke gate")
    source_dist = args.repo_root.resolve() / "tools/session_coordinator/web/dist"
    if not (source_dist / "index.html").is_file():
        raise SystemExit("control console production build is missing")

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        repo = init_repo(root / "repo")
        dist = repo / "tools/session_coordinator/web/dist"
        shutil.copytree(source_dist, dist)
        config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
        with RunningCoordinator.start(config) as running:
            client = CoordinatorClient.from_runtime(config)
            client.command("session.register", {"session_id": "smoke-session"})
            ticket = client.issue_ui_ticket(actor="smoke")
            jar = http.cookiejar.CookieJar()
            opener = urllib.request.build_opener(
                urllib.request.HTTPCookieProcessor(jar), _NoRedirectHandler()
            )
            bootstrap = opener.open(f"{running.base_url}{ticket['bootstrapPath']}", timeout=3)
            if bootstrap.status != 303:
                raise AssertionError("bootstrap did not redirect")
            bootstrap.close()
            page = opener.open(f"{running.base_url}/ui/workflows/smoke", timeout=3)
            html = page.read().decode("utf-8")
            if "Zircon" not in html:
                raise AssertionError("SPA fallback did not serve the control console")
            page.close()
            if '<base href="/ui/"' not in html:
                raise AssertionError("console asset base is not stable for deep links")
            references = re.findall(r'(?:src|href)="\./(assets/[^"]+)"', html)
            if not references:
                raise AssertionError("console index did not reference production assets")
            for reference in references:
                asset = opener.open(f"{running.base_url}/ui/{reference}", timeout=3)
                if asset.headers["Cache-Control"] != "public,max-age=31536000,immutable":
                    raise AssertionError("production asset was not immutable")
                if not asset.read(1):
                    raise AssertionError("production asset was empty")
                asset.close()
            request = urllib.request.Request(
                f"{running.base_url}/control/v1/meta",
                headers={"Origin": running.base_url},
            )
            meta = json.loads(opener.open(request, timeout=3).read())
            if meta["data"]["mutationEnabled"] is not False:
                raise AssertionError("observer surface unexpectedly enables mutation")
            mutation = urllib.request.Request(
                f"{running.base_url}/control/v1/meta",
                method="PATCH",
                headers={"Origin": running.base_url},
            )
            try:
                opener.open(mutation, timeout=3)
            except urllib.error.HTTPError as rejected:
                body = json.loads(rejected.read())
                rejected.close()
                rejection = (rejected.code, body["error"]["code"])
                if rejection not in {(403, "csrf_invalid"), (404, "not_found")}:
                    raise
            else:
                raise AssertionError("control mutation method was unexpectedly accepted")
            if args.controlled_actions:
                _verify_controlled_actions(opener, client, running.base_url, config)
    print(
        "controlled-actions control-console smoke passed"
        if args.controlled_actions
        else "read-only control-console smoke passed"
    )
    return 0


def _verify_controlled_actions(opener, client, base_url: str, config) -> None:
    grant = client.issue_elevation_grant(
        actor="smoke", role="operator", session_id="smoke-session"
    )
    elevated = _browser_json(
        opener,
        base_url,
        "POST",
        "/control/v1/auth/elevate",
        {"grant": grant["grant"]},
    )["data"]
    if elevated["role"] != "operator" or elevated["boundSessionId"] != "smoke-session":
        raise AssertionError("one-use elevation did not bind the browser Session")
    csrf = elevated["csrfToken"]
    catalog = _browser_json(opener, base_url, "GET", "/control/v1/actions/catalog")["data"]
    red = [item for item in catalog["actions"] if item["risk"] == "red"]
    enabled_red = {item["kind"] for item in red if item["enabled"]}
    if enabled_red != {
        "milestone.commit",
        "session.complete",
        "service.stop",
        "service.restart",
        "service.force_stop",
    }:
        raise AssertionError("the closed catalog exposed an unexpected enabled red action")
    if not any(item["kind"] == "maintenance.cleanup" and not item["enabled"] for item in red):
        raise AssertionError("the unimplemented maintenance action was not visibly disabled")
    denied_red = urllib.request.Request(
        f"{base_url}/control/v1/actions/preview",
        data=json.dumps(
            {"kind": "service.stop", "parameters": {"timeoutSeconds": 30}}
        ).encode("utf-8"),
        method="POST",
        headers={
            "Origin": base_url,
            "Content-Type": "application/json",
            "X-CSRF-Token": csrf,
        },
    )
    try:
        opener.open(denied_red, timeout=5)
    except urllib.error.HTTPError as rejected:
        body = json.loads(rejected.read())
        rejected.close()
        if (rejected.code, body["error"]["code"]) != (403, "action_permission_denied"):
            raise
    else:
        raise AssertionError("operator elevation unexpectedly previewed a maintainer red action")
    preview = _browser_json(
        opener,
        base_url,
        "POST",
        "/control/v1/actions/preview",
        {"kind": "session.heartbeat", "parameters": {"sessionId": "smoke-session"}},
        csrf=csrf,
    )["data"]["action"]
    confirmed = _browser_json(
        opener,
        base_url,
        "POST",
        f"/control/v1/actions/{preview['actionId']}/confirm",
        {"phrase": preview["confirmationPhrase"], "reason": "controlled-action smoke acceptance"},
        csrf=csrf,
    )["data"]["action"]
    if confirmed["status"] != "succeeded":
        raise AssertionError("typed controlled action did not succeed")
    with closing(sqlite3.connect(config.database_path)) as connection:
        approval = connection.execute(
            "SELECT reason FROM action_approvals WHERE action_id = ?",
            (preview["actionId"],),
        ).fetchone()
    if approval != ("controlled-action smoke acceptance",):
        raise AssertionError("controlled action approval audit was not persisted")


def _browser_json(opener, base_url, method, path, payload=None, *, csrf=None):
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    headers = {"Origin": base_url, "Content-Type": "application/json"}
    if csrf:
        headers["X-CSRF-Token"] = csrf
    request = urllib.request.Request(
        f"{base_url}{path}", data=data, method=method, headers=headers
    )
    with opener.open(request, timeout=5) as response:
        return json.loads(response.read())


class _NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def http_error_303(self, request, response, code, message, headers):
        return response


if __name__ == "__main__":
    raise SystemExit(main())
