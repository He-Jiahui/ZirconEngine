"""Durable, idempotent notification delivery for Jenkins workflows.

The outbox deliberately lives in the shared :class:`~tools.jenkins.state.State`
records table.  A notification is keyed by ``eventId`` and ``destinationId``;
delivery is never inferred from a Git operation and an ambiguous transport
failure is kept as ``unknown`` until an explicit reconciliation says that a
retry is safe.
"""

from __future__ import annotations

import time
import re
import uuid
from dataclasses import dataclass
from typing import Any, Callable

from ..contracts import JenkinsError, digest, identifier

OUTBOX_DOMAIN = "notification_outbox"
_SECRET = re.compile(r"(?i)(?:https?://[^\s]+|(?:token|secret|key|webhook)\s*[=:]\s*[^\s,;]+)")


@dataclass(frozen=True, slots=True)
class NotificationRecord:
    event_id: str
    destination_id: str
    payload: dict[str, Any]
    payload_digest: str
    status: str
    attempts: int
    last_error: str | None
    created_at: float
    updated_at: float
    claim_epoch: str | None = None

    @classmethod
    def from_record(cls, record: dict) -> "NotificationRecord":
        value = record["payload"]
        return cls(
            event_id=value["eventId"],
            destination_id=value["destinationId"],
            payload=value["payload"],
            payload_digest=value["payloadDigest"],
            status=value["status"],
            attempts=value["attempts"],
            last_error=value.get("lastError"),
            created_at=value["createdAt"],
            updated_at=value["updatedAt"],
            claim_epoch=value.get("claimEpoch"),
        )

    def as_dict(self) -> dict[str, Any]:
        return {
            "eventId": self.event_id,
            "destinationId": self.destination_id,
            "payload": self.payload,
            "payloadDigest": self.payload_digest,
            "status": self.status,
            "attempts": self.attempts,
            "lastError": self.last_error,
            "createdAt": self.created_at,
            "updatedAt": self.updated_at,
            "claimEpoch": self.claim_epoch,
        }


class NotificationOutbox:
    """Persist and deliver notifications with explicit ambiguity handling.

    ``sender`` receives the JSON-compatible payload and returns a truthy value
    after the destination has acknowledged it.  A false result is a known
    non-delivery and may be retried.  Exceptions are ambiguous: the sender may
    have accepted the message before the exception was raised, so the record is
    left ``unknown`` and must be reconciled before another send.
    """

    def __init__(self, state: Any, sender: Callable[[dict[str, Any]], Any] | None = None):
        self.state = state
        self.sender = sender

    @staticmethod
    def _key(event_id: str, destination_id: str) -> str:
        return digest({"eventId": event_id, "destinationId": destination_id})

    def enqueue(self, event_id: str, destination_id: str, payload: dict[str, Any]) -> NotificationRecord:
        event_id = identifier(event_id, "eventId")
        destination_id = identifier(destination_id, "destinationId")
        if not isinstance(payload, dict):
            raise JenkinsError("notification_payload_invalid", "Notification payload must be an object")
        key = self._key(event_id, destination_id)
        now = time.time()
        value = {
            "eventId": event_id,
            "destinationId": destination_id,
            "payload": payload,
            "payloadDigest": digest(payload),
            "status": "pending",
            "attempts": 0,
            "lastError": None,
            "createdAt": now,
            "updatedAt": now,
        }
        with self.state.transaction() as connection:
            old = self.state.get(OUTBOX_DOMAIN, key, connection=connection)
            if old is not None:
                if old["payload"]["payloadDigest"] != value["payloadDigest"]:
                    raise JenkinsError("notification_payload_mismatch", "Notification payload changed")
                return NotificationRecord.from_record(old)
            self.state.put(OUTBOX_DOMAIN, key, value, connection=connection)
            self.state.event("notification_enqueued", {"eventId": event_id, "destinationId": destination_id}, connection=connection)
        return NotificationRecord.from_record({"payload": value})

    def get(self, event_id: str, destination_id: str) -> NotificationRecord | None:
        record = self.state.get(OUTBOX_DOMAIN, self._key(identifier(event_id, "eventId"), identifier(destination_id, "destinationId")))
        return NotificationRecord.from_record(record) if record else None

    def deliver(self, event_id: str, destination_id: str, sender: Callable[[dict[str, Any]], Any] | None = None) -> NotificationRecord:
        record = self.get(event_id, destination_id)
        if record is None:
            raise JenkinsError("notification_missing", "Notification is not queued")
        if record.status == "delivered":
            return record
        if record.status == "unknown":
            raise JenkinsError("notification_delivery_ambiguous", "Reconcile notification before retrying")
        if record.status == "delivering":
            raise JenkinsError("notification_delivery_in_progress", "Notification delivery is already in progress", retryable=True)
        if record.status not in {"pending", "retryable"}:
            raise JenkinsError("notification_not_deliverable", "Notification is not deliverable")
        sender = sender or self.sender
        if sender is None:
            raise JenkinsError("notification_sender_missing", "A notification sender is required")
        key = self._key(record.event_id, record.destination_id)
        now = time.time()
        claim_epoch = uuid.uuid4().hex
        in_flight = {**record.as_dict(), "status": "delivering", "attempts": record.attempts + 1,
                     "claimEpoch": claim_epoch, "updatedAt": now}
        with self.state.transaction() as connection:
            current = self.state.get(OUTBOX_DOMAIN, key, connection=connection)
            if current is None or current["payload"]["payloadDigest"] != record.payload_digest:
                raise JenkinsError("notification_state_conflict", "Notification changed before delivery", retryable=True)
            if current["payload"]["status"] not in {"pending", "retryable"}:
                raise JenkinsError("notification_delivery_in_progress", "Notification claim already exists", retryable=True)
            self.state.put(OUTBOX_DOMAIN, key, in_flight, expected_version=current["version"], connection=connection)
        try:
            acknowledged = bool(sender(record.payload))
        except Exception as error:
            return self._finish(key, claim_epoch, "unknown", self._sanitize(error))
        return self._finish(key, claim_epoch, "delivered" if acknowledged else "retryable", None if acknowledged else "destination rejected")

    def reconcile(self, event_id: str, destination_id: str, probe: Callable[[dict[str, Any]], Any]) -> NotificationRecord:
        record = self.get(event_id, destination_id)
        if record is None:
            raise JenkinsError("notification_missing", "Notification is not queued")
        if record.status == "delivered":
            return record
        if record.status != "unknown":
            raise JenkinsError("notification_not_ambiguous", "Only unknown delivery may be reconciled")
        try:
            result = probe(record.payload)
        except Exception as error:
            return self._finish(self._key(record.event_id, record.destination_id), record.claim_epoch or "", "unknown", self._sanitize(error))
        if result is True:
            return self._finish(self._key(record.event_id, record.destination_id), record.claim_epoch or "", "delivered", None)
        if result is False:
            return self._finish(self._key(record.event_id, record.destination_id), record.claim_epoch or "", "retryable", "reconciliation found no delivery")
        return record

    def recover_delivering(self) -> tuple[str, ...]:
        recovered = []
        with self.state.transaction() as connection:
            for row in self.state.list(OUTBOX_DOMAIN, connection=connection):
                if row["payload"]["status"] != "delivering":
                    continue
                value = {**row["payload"], "status": "unknown", "claimEpoch": uuid.uuid4().hex,
                         "lastError": "delivery interrupted; reconciliation required", "updatedAt": time.time()}
                self.state.put(OUTBOX_DOMAIN, row["key"], value, expected_version=row["version"], connection=connection)
                recovered.append(value["eventId"])
        return tuple(recovered)

    @staticmethod
    def _sanitize(error: Exception | str) -> str:
        return _SECRET.sub("[redacted]", str(error)).strip()[:500] or "notification delivery failed"

    def _finish(self, key: str, claim_epoch: str, status: str, error: str | None) -> NotificationRecord:
        try:
            with self.state.transaction() as connection:
                current = self.state.get(OUTBOX_DOMAIN, key, connection=connection)
                if current is None:
                    raise JenkinsError("notification_missing", "Notification is not queued")
                if current["payload"].get("claimEpoch") != claim_epoch:
                    raise JenkinsError("notification_claim_stale", "Notification claim is stale and no longer current", retryable=True)
                value = {**current["payload"], "status": status, "lastError": error, "updatedAt": time.time()}
                self.state.put(OUTBOX_DOMAIN, key, value, expected_version=current["version"], connection=connection)
                updated = self.state.get(OUTBOX_DOMAIN, key, connection=connection)
            return NotificationRecord.from_record(updated)
        except JenkinsError:
            raise
        except Exception as state_error:
            # The sender may already have accepted the event; preserve ambiguity.
            raise JenkinsError("notification_state_unknown", "Notification result could not be persisted", retryable=True, details={"error": "persistence failure"}) from state_error


def handle(action: str, payload: dict[str, Any], state: Any, repo_root: str | None = None, *, domain: str = OUTBOX_DOMAIN) -> dict[str, Any]:
    """Small package entry point used by Jenkins steps and adapters."""
    outbox = NotificationOutbox(state)
    if domain != OUTBOX_DOMAIN:
        raise JenkinsError("notification_domain_invalid", "Unsupported notification domain")
    if action in {"enqueue", "deliver", "reconcile"}:
        repository_id = payload.get("repositoryId")
        session_id = payload.get("sessionId")
        destination_id = payload.get("destinationId")
        if not repository_id or not session_id or payload.get("destinationProof") != destination_id:
            raise JenkinsError("notification_authorization_missing", "Notification action requires scoped notify authorization and destination proof")
        state.check_authorization(repository_id, session_id, "notify", [])
    if action == "enqueue":
        return outbox.enqueue(payload["eventId"], payload["destinationId"], payload.get("payload", {})).as_dict()
    if action == "query":
        record = outbox.get(payload["eventId"], payload["destinationId"])
        return record.as_dict() if record else {"status": "missing"}
    if action in {"deliver", "reconcile"}:
        # Adapters must register a sender/probe on the process; arbitrary
        # endpoint JSON in a Jenkins request is intentionally rejected.
        adapter = getattr(state, "notification_sender", None) if action == "deliver" else getattr(state, "notification_probe", None)
        if adapter is None:
            raise JenkinsError("notification_not_configured", "Notification endpoint is not configured", retryable=True)
        if action == "deliver":
            return outbox.deliver(payload["eventId"], payload["destinationId"], adapter).as_dict()
        return outbox.reconcile(payload["eventId"], payload["destinationId"], adapter).as_dict()
    raise JenkinsError("notification_action_invalid", "Action requires a configured sender or probe adapter")


__all__ = ["NotificationOutbox", "NotificationRecord", "OUTBOX_DOMAIN", "handle"]
