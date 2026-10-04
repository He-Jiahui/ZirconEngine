"""Hash-guarded compensation before an incremental patch flow is registered."""
from __future__ import annotations

from pathlib import Path

from .contracts import JenkinsError, digest, file_digest, response
from .resources import canonical_build_root, physical_path_under
from .source import _atomic_write, _before_hash, canonical_path, repository_id


def _active_execution_reference(state, operation_id: str, preparation: dict | None) -> bool:
    """Return whether managed execution state still points at this patch.

    A flow record is not the only durable reference to a sealed input.  The
    execution broker creates consumer and native-job records after a CPS
    handoff, so recovery must inspect those records before restoring the
    shared checkout.  Matching the operation id or the sealed source digest
    is deliberate: either binding is sufficient to make compensation unsafe.
    """
    refs = {operation_id}
    if isinstance(preparation, dict):
        result = preparation.get("result") or {}
        for key in ("sealedInputRef", "sourceInputDigest", "sourceDigest"):
            value = result.get(key)
            if isinstance(value, str) and value:
                refs.add(value)

    def matches(payload: dict, seen: set[str] | None = None) -> bool:
        seen = set() if seen is None else seen
        for key in ("patchOperationId", "patchOperationRef", "operationId",
                    "sealedInputRef", "sourceInputDigest", "sourceDigest",
                    "inputDigest"):
            value = payload.get(key)
            if isinstance(value, str) and value in refs:
                return True
        execution_id = payload.get("executionId")
        if isinstance(execution_id, str) and execution_id not in seen:
            seen.add(execution_id)
            execution = state.get("execution", execution_id)
            if execution and matches(execution["payload"], seen):
                return True
        return False

    for row in state.list("execution_consumer"):
        payload = row["payload"]
        if payload.get("status") == "active" and matches(payload):
            return True
    for row in state.list("native_job"):
        payload = row["payload"]
        if payload.get("status") not in {"terminal", "complete", "released"} and matches(payload):
            return True
    return False


def _compensate(state, root, intent, ref):
    owner = intent['identity']['attemptId']
    operation = state.get_operation('patch-' + ref)
    # The process lock proves the source writer has stopped.  Managed native
    # execution can only be submitted by a registered workflow; its absence is
    # verified by the caller, never asserted by a client-supplied flag.
    proof = {'kind': 'no-execution-dispatch', 'workflowRegistered': False, 'patchLockHeld': True}
    if operation is None:
        return {'status': 'complete', 'operations': [], 'terminationEvidence': proof}
    patch = operation['payload']
    if operation['kind'] != 'source_patch' or patch.get('owner') != owner or patch.get('beforeHashes') != intent['beforeHashes'] or patch.get('patchDigest') != intent['renderedDigest']:
        raise JenkinsError('patch_identity_mismatch', 'Compensation journal differs from the immutable patch intent')
    if patch.get('repositoryRoot', '').casefold() != str(root).casefold():
        raise JenkinsError('patch_repository_mismatch', 'Compensation belongs to a different checkout')
    before, after = patch['beforeHashes'], patch.get('expectedAfterHashes')
    if not isinstance(after, dict) or set(before) != set(after):
        raise JenkinsError('patch_after_missing', 'Compensation requires complete postimage hashes')
    for workflow in state.list('workflow'):
        if operation['operationId'] in workflow['payload'].get('patchOperationRefs', []):
            raise JenkinsError('shared_source_active', 'Another registered flow references this patch')
    preparation = state.get('patch_preparation', ref)
    if _active_execution_reference(state, operation['operationId'],
                                   preparation['payload'] if preparation else None):
        raise JenkinsError('shared_source_active',
                           'Managed execution still references this patch; compensation is unsafe')
    repo = repository_id(root)
    claims = {row['payload']['path']: row for row in state.list('path_claims')
              if row['payload'].get('repositoryId') == repo}
    selected = canonical_build_root(intent['buildRoot'])
    object_root = physical_path_under(selected, patch.get('objectRoot', ''))
    if object_root != selected.path / 'zircon-jenkins':
        raise JenkinsError('object_root_mismatch', 'Original patch objects must retain their registered storage root')
    originals = {}
    observed = {}
    for path, checksum in before.items():
        canonical_path(root, path)
        current = _before_hash(root, path)
        observed[path] = current
        if current not in {checksum, after[path]}:
            raise JenkinsError('foreign_edit_during_compensation', 'Later source edits prevent patch compensation', details={'path': path})
        for claimed_path, row in claims.items():
            if (claimed_path == path or claimed_path.startswith(path + '/') or path.startswith(claimed_path + '/')) and row['payload']['owner'] != owner:
                raise JenkinsError('foreign_claim_during_compensation', 'Another source owner prevents compensation')
        if checksum is None:
            originals[path] = None
        else:
            if patch.get('beforeObjects', {}).get(path) != checksum:
                raise JenkinsError('before_object_missing', 'Patch journal has no immutable original object')
            original = physical_path_under(selected, object_root / 'inputs/objects' / checksum)
            if not original.is_file() or file_digest(original) != checksum:
                raise JenkinsError('before_object_corrupt', 'Original patch bytes are missing or corrupt')
            originals[path] = original.read_bytes()
    compensation_id = digest({'patch': operation['operationId'], 'recoveryOwner': owner})
    journal = state.get_operation(compensation_id)
    if journal and journal['status'] == 'complete':
        return {'status': 'complete', 'operations': [compensation_id], 'terminationEvidence': proof}
    if journal and journal['status'] == 'blocked':
        raise JenkinsError('compensation_blocked', 'An earlier compensation stopped at a foreign edit')
    if operation['status'] == 'prepared':
        state.transition_operation(operation['operationId'], 'prepared', 'aborted',
                                   {'observedHashes': observed, 'reasonCode': 'preparation_cancelled',
                                    'terminationEvidence': proof})
    if journal is None:
        journal = state.put_operation('source_compensation', compensation_id,
            {'patchOperationId': operation['operationId'], 'owner': owner, 'repositoryRoot': str(root),
             'paths': sorted(before), 'beforeHashes': before, 'afterHashes': after, 'terminationEvidence': proof})
    restored = list((journal.get('result') or {}).get('restoredPaths', []))
    try:
        for path in sorted(before):
            current = _before_hash(root, path)
            if current != before[path]:
                if current != after[path]:
                    raise JenkinsError('foreign_edit_during_compensation', 'Source changed during compensation', details={'path': path})
                target = root.joinpath(*path.split('/'))
                if originals[path] is None:
                    target.unlink(missing_ok=True)
                else:
                    _atomic_write(target, originals[path])
                if _before_hash(root, path) != before[path]:
                    raise JenkinsError('foreign_edit_during_compensation', 'Restored source changed before receipt')
            if path not in restored:
                restored.append(path)
            state.transition_operation(compensation_id, 'prepared', 'prepared', {'restoredPaths': restored})
        with state.transaction() as connection:
            for path, row in claims.items():
                if path in before and row['payload']['owner'] == owner:
                    state.delete('path_claims', row['key'], expected_version=row['version'], connection=connection)
            state.transition_operation(compensation_id, 'prepared', 'complete',
                {'restoredPaths': restored, 'terminationEvidence': proof}, connection=connection)
        return {'status': 'complete', 'operations': [compensation_id], 'terminationEvidence': proof}
    except BaseException as error:
        state.transition_operation(compensation_id, 'prepared', 'blocked',
            {'restoredPaths': restored, 'reasonCode': getattr(error, 'code', type(error).__name__)})
        raise


def reconcile_preparation(state, root: Path, intent: dict, ref: str):
    identity = intent['identity']
    workflow_key = f"{identity['sessionId']}:{identity['requestId']}:{identity['attemptId']}"
    if state.get('workflow', workflow_key):
        from .workflow.recovery import recover_flow
        return recover_flow(state, root, workflow_key, error='incremental_patch_reconciliation')
    preparation = state.get('patch_preparation', ref)
    if preparation and preparation['payload'].get('status') == 'prepared':
        # A successful preparation may already have returned to another flow
        # invocation.  Its dispatch must be reconciled before undoing source.
        return response('blocked', recovery={'status': 'blocked',
                        'reasonCode': 'prepared_flow_dispatch_unreconciled'})
    if preparation and preparation['payload'].get('recovery'):
        recovery = preparation['payload']['recovery']
        if recovery['status'] in {'complete', 'blocked'}:
            return response('failed' if recovery['status'] == 'complete' else 'blocked', recovery=recovery)
    try:
        from .patches import _head
        if _head(root) != intent['baseHead']:
            raise JenkinsError('foreign_head_change', 'Git HEAD changed before compensation')
        recovery = _compensate(state, root, intent, ref)
    except JenkinsError as error:
        recovery = {'status': 'waiting' if error.retryable else 'blocked', 'reasonCode': error.code}
    value = {**(preparation['payload'] if preparation else {}), 'status': 'failed', 'recovery': recovery}
    state.put('patch_preparation', ref, value, expected_version=preparation['version'] if preparation else 0)
    if recovery['status'] == 'complete':
        for row in state.list('sealed_consumer'):
            consumer = row['payload']
            if consumer.get('repositoryId') == identity['repositoryId'] and consumer.get('sessionId') == identity['sessionId'] and consumer.get('owner') == identity['attemptId']:
                state.put('sealed_consumer', row['key'], {**consumer, 'status': 'released'}, expected_version=row['version'])
        request = state.get_request(identity['repositoryId'], identity['sessionId'], identity['requestId'])
        if request and request['status'] not in {'failed', 'cancelled', 'accepted'}:
            state.transition_request(identity['repositoryId'], identity['sessionId'], identity['requestId'], 1,
                                     request['status'], 'failed', {'recovery': recovery})
    return response('failed' if recovery['status'] == 'complete' else 'blocked', recovery=recovery)
