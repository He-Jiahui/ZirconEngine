from __future__ import annotations

import argparse
import http.cookiejar
import json
import re
import shutil
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
    args = parser.parse_args()
    if not args.read_only_console:
        parser.error("--read-only-console is required")
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
                if rejected.code != 404 or body["error"]["code"] != "not_found":
                    raise
            else:
                raise AssertionError("control mutation method was unexpectedly accepted")
    print("read-only control-console smoke passed")
    return 0


class _NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def http_error_303(self, request, response, code, message, headers):
        return response


if __name__ == "__main__":
    raise SystemExit(main())
