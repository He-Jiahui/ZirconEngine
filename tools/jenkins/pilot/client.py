"""Bounded Jenkins HTTP client with durable reconciliation of accepted requests."""

from __future__ import annotations

import base64
import hashlib
import http.cookiejar
import json
import re
import secrets
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Mapping

from .contracts import PilotError, RequestIdentity, identifier
from .journal import SubmissionJournal
from .snapshot import load_snapshot

MAX_RESPONSE_BYTES = 16 * 1024 * 1024
MAX_BUNDLE_BYTES = 32 * 1024 * 1024


class JenkinsTransportError(PilotError):
    def __init__(self, message: str, *, http_status: int | None = None):
        super().__init__(message)
        self.http_status = http_status


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class JenkinsClient:
    def __init__(
        self, url: str, username: str, password: str, *, timeout: float = 15
    ) -> None:
        parsed = urllib.parse.urlsplit(url)
        if (
            parsed.scheme != "http" or parsed.hostname != "127.0.0.1"
            or parsed.username or parsed.password or parsed.query or parsed.fragment
            or parsed.path not in {"", "/"} or not parsed.port
        ):
            raise PilotError("the isolated pilot requires an explicit loopback Jenkins URL")
        self.url = url.rstrip("/")
        self._authorization = "Basic " + base64.b64encode(
            f"{username}:{password}".encode("utf-8")
        ).decode("ascii")
        self.timeout = timeout
        self.opener = urllib.request.build_opener(
            _NoRedirect(), urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar())
        )

    def request(
        self, path: str, *, data: bytes | None = None, content_type: str | None = None,
        post: bool = False, crumb: bool = True,
    ) -> tuple[bytes, Mapping[str, str], int]:
        if not path.startswith("/") or path.startswith("//") or "\0" in path:
            raise PilotError("Jenkins endpoint must be a relative absolute-path reference")
        headers = {"Authorization": self._authorization}
        if content_type:
            headers["Content-Type"] = content_type
        if post and crumb:
            # API-token installations may omit crumbs; the pilot's private password does not.
            try:
                payload = self.api("/crumbIssuer/api/json")
                headers[str(payload["crumbRequestField"])] = str(payload["crumb"])
            except JenkinsTransportError as error:
                raise JenkinsTransportError("cannot obtain Jenkins CSRF crumb") from error
        req = urllib.request.Request(
            self.url + path, data=data, headers=headers, method="POST" if post else "GET"
        )
        try:
            with self.opener.open(req, timeout=self.timeout) as response:
                content = response.read(MAX_RESPONSE_BYTES + 1)
                if len(content) > MAX_RESPONSE_BYTES:
                    raise JenkinsTransportError("Jenkins response exceeds the pilot limit")
                return content, dict(response.headers), response.status
        except urllib.error.HTTPError as error:
            if error.code in {301, 302, 303}:
                return b"", dict(error.headers), error.code
            raise JenkinsTransportError(f"Jenkins endpoint returned HTTP {error.code}", http_status=error.code) from None
        except (urllib.error.URLError, OSError, TimeoutError):
            raise JenkinsTransportError("Jenkins transport unavailable; reconcile the existing request") from None

    def api(self, path: str) -> Mapping[str, object]:
        raw, _headers, status = self.request(path)
        if status != 200:
            raise JenkinsTransportError("Jenkins API did not return a JSON response")
        try:
            result = json.loads(raw)
        except (ValueError, UnicodeError):
            raise JenkinsTransportError("Jenkins API response is invalid JSON") from None
        if not isinstance(result, dict):
            raise JenkinsTransportError("Jenkins API response is not an object")
        return result

    def healthy(self) -> bool:
        try:
            return "mode" in self.api("/api/json?tree=mode")
        except JenkinsTransportError:
            return False

    @staticmethod
    def _multipart(parameters: Mapping[str, str], bundle: bytes) -> tuple[bytes, str]:
        boundary = "zircon-pilot-" + secrets.token_hex(16)
        parts: list[bytes] = []
        for name, value in parameters.items():
            parts.append(
                f'--{boundary}\r\nContent-Disposition: form-data; name="{name}"\r\n\r\n{value}\r\n'.encode()
            )
        parts.append(
            (
                f'--{boundary}\r\nContent-Disposition: form-data; name="SOURCE_BUNDLE"; filename="source.zip"\r\n'
                "Content-Type: application/zip\r\n\r\n"
            ).encode()
        )
        parts.extend([bundle, f"\r\n--{boundary}--\r\n".encode()])
        return b"".join(parts), f"multipart/form-data; boundary={boundary}"

    def submit(
        self, job: str, identity: RequestIdentity, bundle: Path, journal: SubmissionJournal
    ) -> Mapping[str, object]:
        identifier(job, "job")
        sealed = load_snapshot(bundle, expected_input_hash=identity.input_hash)
        payload = bundle.read_bytes()
        bundle_hash = hashlib.sha256(payload).hexdigest()
        if bundle_hash != sealed.bundle_hash:
            raise PilotError("source bundle changed before upload")
        record, dispatch = journal.reserve(identity, bundle_hash, job)
        if not dispatch:
            return self.reconcile(job, identity, journal)
        parameters = identity.parameters()
        parameters["BUNDLE_HASH"] = bundle_hash
        body, content_type = self._multipart(parameters, payload)
        journal.update(identity, "uncertain")
        try:
            _raw, headers, status = self.request(
                f"/job/{job}/buildWithParameters", data=body, content_type=content_type, post=True
            )
        except JenkinsTransportError:
            # An accepted POST may have lost its response. This identity is never re-dispatched.
            return journal.get(identity)
        location = headers.get("Location", headers.get("location", ""))
        target = urllib.parse.urlsplit(location)
        expected = urllib.parse.urlsplit(self.url)
        if target.netloc and (target.scheme, target.netloc) != (expected.scheme, expected.netloc):
            raise PilotError("Jenkins returned a foreign queue location")
        match = re.fullmatch(r"/queue/item/([1-9][0-9]*)/?", target.path)
        if status != 201 or not match:
            raise JenkinsTransportError("submission response has no exact Jenkins queue identity")
        return journal.update(identity, "queued", queue_id=int(match.group(1)))

    @staticmethod
    def _parameters(record: Mapping[str, object]) -> dict[str, object]:
        result: dict[str, object] = {}
        for action in record.get("actions", []):
            if not isinstance(action, dict):
                continue
            for parameter in action.get("parameters", []):
                if isinstance(parameter, dict) and isinstance(parameter.get("name"), str):
                    if parameter["name"] in result:
                        raise PilotError("Jenkins request has duplicate parameter names")
                    result[parameter["name"]] = parameter.get("value")
        return result

    @classmethod
    def _matches(
        cls, record: Mapping[str, object], identity: RequestIdentity, bundle_hash: str
    ) -> bool:
        parameters = cls._parameters(record)
        if parameters.get("REQUEST_ID") != identity.request_id or parameters.get("SESSION_ID") != identity.session_id:
            return False
        expected = {**identity.parameters(), "BUNDLE_HASH": bundle_hash}
        if any(str(parameters.get(key, "")) != value for key, value in expected.items()):
            raise PilotError("Jenkins request identity conflicts with its immutable journal binding")
        return True

    def reconcile(
        self, job: str, identity: RequestIdentity, journal: SubmissionJournal
    ) -> Mapping[str, object]:
        identifier(job, "job")
        record = journal.get(identity)
        if record["job"] != job:
            raise PilotError("request belongs to a different Jenkins job")
        if record["state"] in {"completed", "cancelled"}:
            return record
        if record["build_number"] is not None:
            build = self.api(
                f"/job/{job}/{record['build_number']}/api/json?tree=number,building,result,actions[parameters[name,value]]"
            )
            if not self._matches(build, identity, str(record["bundle_hash"])):
                raise PilotError("bound Jenkins build no longer contains the expected request")
            return self._record_build(identity, build, journal)
        if record["queue_id"] is not None:
            try:
                item = self.api(f"/queue/item/{record['queue_id']}/api/json?tree=id,cancelled,task[name],actions[parameters[name,value]],executable[number]")
            except JenkinsTransportError as error:
                if error.http_status != 404:
                    raise
            else:
                if item.get("id") != record["queue_id"] or item.get("task", {}).get("name") != job or not self._matches(item, identity, str(record["bundle_hash"])):
                    raise PilotError("bound Jenkins queue item no longer contains the expected request")
                if item.get("cancelled") is True:
                    return journal.update(identity, "cancelled", result="CANCELLED")
                executable = item.get("executable", {}).get("number")
                if executable is not None:
                    build = self.api(f"/job/{job}/{executable}/api/json?tree=number,building,result,actions[parameters[name,value]]")
                    if not self._matches(build, identity, str(record["bundle_hash"])):
                        raise PilotError("queue executable does not contain the immutable request")
                    return self._record_build(identity, build, journal)
        builds = self.api(
            f"/job/{job}/api/json?tree=builds[number,building,result,actions[parameters[name,value]]]{{0,100}}"
        ).get("builds", [])
        matches = [b for b in builds if isinstance(b, dict) and self._matches(b, identity, str(record["bundle_hash"]))]
        queue = self.api(
            "/queue/api/json?tree=items[id,cancelled,task[name],actions[parameters[name,value]],executable[number]]"
        ).get("items", [])
        queued = [q for q in queue if isinstance(q, dict) and q.get("task", {}).get("name") == job
                  and self._matches(q, identity, str(record["bundle_hash"]))]
        if len(matches) > 1 or len(queued) > 1:
            raise PilotError("multiple Jenkins executions exist for one immutable request")
        if matches:
            if queued and queued[0].get("executable", {}).get("number") != matches[0].get("number"):
                raise PilotError("a second Jenkins queue entry exists for the bound request")
            return self._record_build(identity, matches[0], journal)
        if queued:
            return journal.update(identity, "cancelled" if queued[0].get("cancelled") else "queued", queue_id=int(queued[0]["id"]))
        # Missing observation is not proof of a terminal job or permission to submit again.
        return record

    @staticmethod
    def _record_build(
        identity: RequestIdentity, build: Mapping[str, object], journal: SubmissionJournal
    ) -> Mapping[str, object]:
        building = build.get("building")
        result = build.get("result")
        terminal = building is False and isinstance(result, str) and bool(result)
        return journal.update(
            identity, "completed" if terminal else "running",
            build_number=build["number"], result=result if terminal else None,
        )

    def cancel(self, job: str, identity: RequestIdentity, journal: SubmissionJournal) -> Mapping[str, object]:
        record = self.reconcile(job, identity, journal)
        if record["state"] in {"completed", "cancelled"}:
            return record
        if record["build_number"] is not None:
            self.request(f"/job/{job}/{record['build_number']}/stop", post=True, data=b"")
        elif record["queue_id"] is not None:
            self.request(f"/queue/cancelItem?id={record['queue_id']}", post=True, data=b"")
        else:
            raise PilotError("request has no verified Jenkins queue or build to cancel")
        return self.reconcile(job, identity, journal)

    def receipt(self, job: str, build_number: int) -> Mapping[str, object]:
        identifier(job, "job")
        if type(build_number) is not int or build_number < 1:
            raise PilotError("build number must be a positive integer")
        content, _headers, status = self.request(f"/job/{job}/{build_number}/artifact/receipt.json")
        if status != 200:
            raise PilotError("build has no archived pilot receipt")
        try:
            result = json.loads(content)
        except (ValueError, UnicodeError):
            raise PilotError("build receipt is invalid JSON") from None
        if not isinstance(result, dict):
            raise PilotError("build receipt is not an object")
        return result
