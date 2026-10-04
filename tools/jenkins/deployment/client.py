from __future__ import annotations

import json
import urllib.request
from dataclasses import dataclass


@dataclass(frozen=True)
class JenkinsClient:
    base_url: str
    timeout: float = 5.0

    def get_json(self, path: str) -> dict:
        request = urllib.request.Request(self.base_url.rstrip("/") + "/" + path.lstrip("/"), headers={"Accept": "application/json"})
        with urllib.request.urlopen(request, timeout=self.timeout) as response:
            return json.loads(response.read().decode("utf-8"))

    def healthy(self) -> bool:
        try: self.get_json("api/json")
        except Exception: return False
        return True
