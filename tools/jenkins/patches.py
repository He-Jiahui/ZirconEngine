"""Immutable incremental patch intake, applied and sealed by the Jenkins flow.

Registration records intent without changing the shared checkout.  Preparation
holds an OS process lock and uses the source authority's hash and ownership
gates.  The immutable build request is registered only after sealing succeeds.
"""
from __future__ import annotations

import hashlib
from pathlib import Path
import subprocess
from typing import Mapping

from .contracts import JenkinsError, digest, identifier, response
from .resources import canonical_build_root
from .source import (_before_hash, apply_owned_patch, canonical_path,
                     claim_paths, handle as source_handle, repository_id)
from .source.unified_patch import materialize_patch, parse_unified_patch
from .state import State
from .state.locks import process_lock

_IDENTITY = ('repositoryId', 'sessionId', 'requestId', 'attemptId', 'generation',
             'stageImplementationDigest')
_MAX_PATCH_BYTES = 4 * 1024 * 1024


def _identity(payload: Mapping) -> dict:
    supplied = payload.get('identity', payload)
    if not isinstance(supplied, Mapping):
        raise JenkinsError('identity_missing', 'Patch identity is required')
    value = {key: supplied.get(key) for key in _IDENTITY}
    for key in _IDENTITY[:4]:
        identifier(value[key], key)
    if type(value['generation']) is not int or value['generation'] != 1:
        raise JenkinsError('generation_invalid', 'New patch requests require generation 1')
    implementation = value['stageImplementationDigest']
    if not isinstance(implementation, str) or len(implementation) != 64 or any(c not in '0123456789abcdef' for c in implementation):
        raise JenkinsError('driver_context_missing', 'Patch identity requires a sealed driver digest')
    for key in _IDENTITY:
        if key in payload and key in supplied and payload[key] != supplied[key]:
            raise JenkinsError('identity_conflict', 'Patch control and request identities differ')
    return value


def _lock(root: Path):
    return process_lock(root / '.jenkins/state/patch-locks' / (repository_id(root) + '.lock'),
                        reason='patch_checkout_busy')


def _build_root(root: Path, requested=None) -> str:
    spec_path = root / '.jenkins/deployment-spec.json'
    if spec_path.is_file():
        from .deployment.paths import resolve_paths
        from .deployment.spec import load_spec
        selected = resolve_paths(load_spec(spec_path), requested).build_root
    else:
        selected = canonical_build_root(requested).path
    return str(canonical_build_root(selected).path)


def _head(root: Path) -> str | None:
    if not (root / '.git').exists():
        return None  # A bounded source fixture has no Git database.
    completed = subprocess.run(['git', '-C', str(root), 'rev-parse', '--verify', 'HEAD'],
                               capture_output=True, text=True)
    if completed.returncode:
        raise JenkinsError('git_base_unavailable', 'Patch intake cannot read the checkout HEAD')
    return completed.stdout.strip()


def _scope(root: Path, values: Mapping) -> dict:
    if not isinstance(values, Mapping) or not values:
        raise JenkinsError('patch_scope_mismatch', 'Patch requires exact before hashes')
    result = {}
    for key, checksum in values.items():
        path = canonical_path(root, key)
        if path in result:
            raise JenkinsError('patch_path_collision', 'Patch before hashes contain aliased paths')
        if checksum is not None and (not isinstance(checksum, str) or len(checksum) != 64 or any(c not in '0123456789abcdef' for c in checksum)):
            raise JenkinsError('before_hash_invalid', 'Before hashes must be SHA-256 or null')
        result[path] = checksum
    return result


def _authorize(state, intent):
    identity = intent['identity']
    auth = state.check_authorization(identity['repositoryId'], identity['sessionId'],
                                     'implementation', list(intent['beforeHashes']))
    state.check_authorization(identity['repositoryId'], identity['sessionId'], 'validation',
                              sorted(set(intent['beforeHashes']) | set(intent['declaredDependencies'])))
    if intent.get('authorizationSnapshot') not in (None, auth['snapshotDigest']):
        raise JenkinsError('patch_authorization_changed', 'Patch authorization no longer matches its registration')
    return auth


def register_patch_intent(state: State, repo_root: Path, payload: dict, *, trusted_driver_digest: str) -> dict:
    root = repo_root.absolute()
    identity = _identity(payload)
    if identity['stageImplementationDigest'] != trusted_driver_digest:
        raise JenkinsError('driver_context_mismatch', 'Patch intake must use its active sealed driver')
    text = payload.get('unifiedPatch')
    if not isinstance(text, str) or len(text.encode('utf-8')) > _MAX_PATCH_BYTES:
        raise JenkinsError('patch_input_invalid', 'Patch text is required and must fit the bounded intake')
    before = _scope(root, payload.get('beforeHashes'))
    dependencies = payload.get('declaredDependencies', [])
    if not isinstance(dependencies, list) or not all(isinstance(p, str) for p in dependencies):
        raise JenkinsError('patch_dependency_invalid', 'Declared dependencies must be an explicit file list')
    dependencies = sorted({canonical_path(root, p) for p in dependencies} - set(before))
    coverage = payload.get('coverage')
    if not isinstance(coverage, dict) or not coverage or {'sourceDigest', 'coverageDigest'} & set(coverage):
        raise JenkinsError('coverage_missing', 'Patch intake requires coverage before source-derived digests exist')
    external = payload.get('externalInputs', [])
    if not isinstance(external, list) or not all(isinstance(p, dict) for p in external):
        raise JenkinsError('patch_external_input_invalid', 'External inputs must be explicit metadata objects')
    build_root = _build_root(root, payload.get('buildRoot'))
    supplied = {'identity': identity, 'repositoryRoot': str(root), 'unifiedPatch': text,
                'beforeHashes': before, 'declaredDependencies': dependencies, 'coverage': coverage,
                'externalInputs': external, 'baseHead': payload.get('baseHead'), 'buildRoot': build_root}
    input_digest = digest(supplied)
    slot = digest([identity[key] for key in _IDENTITY[:3]])
    owner_slot = digest([repository_id(root), identity['attemptId']])
    with _lock(root):
        prior = state.get('patch_request', slot)
        if prior:
            if prior['payload'].get('inputDigest') != input_digest:
                raise JenkinsError('patch_request_conflict', 'This request already owns a different patch intent')
            intent = _load_intent(state, root, {'identity': identity, 'patchRequestRef': prior['payload']['patchRequestRef']})
            _authorize(state, intent)
            return response('registered', patchRequestRef=prior['payload']['patchRequestRef'],
                            request_id=identity['requestId'], observed_generation=1, reused=True)
        if state.get_request(identity['repositoryId'], identity['sessionId'], identity['requestId']):
            raise JenkinsError('patch_request_conflict', 'Existing source request cannot be replaced by raw patch intake')
        owner_binding = state.get('patch_owner', owner_slot)
        if owner_binding and owner_binding['payload'].get('identity') != identity:
            raise JenkinsError('patch_owner_conflict', 'This patch attempt belongs to another immutable request')
        if not owner_binding and any(row['payload'].get('repositoryId') == repository_id(root)
                                     and row['payload'].get('owner') == identity['attemptId']
                                     for row in state.list('path_claims')):
            raise JenkinsError('patch_owner_conflict', 'An existing source owner cannot be adopted by patch intake')
        parsed = parse_unified_patch(text)
        rendered = materialize_patch(root, parsed, before)
        if set(rendered) != set(before):
            raise JenkinsError('patch_scope_mismatch', 'Parsed patch and before hashes differ')
        dependency_hashes = {}
        for path in dependencies:
            target = root.joinpath(*path.split('/'))
            if not target.is_file():
                raise JenkinsError('patch_dependency_invalid', 'Dependencies must name existing files', details={'path': path})
            dependency_hashes[path] = _before_hash(root, path)
        observed_head = _head(root)
        if supplied['baseHead'] is not None and supplied['baseHead'] != observed_head:
            raise JenkinsError('patch_base_changed', 'Patch base differs from the shared checkout HEAD')
        intent = {**supplied, 'schemaVersion': 1, 'inputDigest': input_digest,
                  'baseHead': observed_head, 'dependencyHashes': dependency_hashes,
                  'renderedPatch': rendered, 'renderedDigest': digest(rendered), 'patchDigest': digest(text)}
        intent['authorizationSnapshot'] = _authorize(state, intent)['snapshotDigest']
        ref = digest(intent)
        with state.transaction() as connection:
            if state.get_request(identity['repositoryId'], identity['sessionId'], identity['requestId'], connection=connection):
                raise JenkinsError('patch_request_conflict', 'Request was registered while patch intake was being checked')
            current = state.get('patch_request', slot, connection=connection)
            if current and current['payload'].get('patchRequestRef') != ref:
                raise JenkinsError('patch_request_conflict', 'Request already belongs to another patch intent')
            state.put('patch_intent', ref, intent, expected_version=0, connection=connection)
            state.put('patch_request', slot, {'patchRequestRef': ref, 'inputDigest': input_digest},
                      expected_version=0, connection=connection)
            state.put('patch_owner', owner_slot, {'identity': identity, 'patchRequestRef': ref},
                      expected_version=0, connection=connection)
        return response('registered', patchRequestRef=ref, request_id=identity['requestId'],
                        observed_generation=1, reused=False)


def _load_intent(state, root, payload):
    ref = payload.get('patchRequestRef')
    identifier(ref, 'patchRequestRef')
    record = state.get('patch_intent', ref)
    if record is None or digest(record['payload']) != ref:
        raise JenkinsError('patch_intent_missing', 'Registered patch intent is missing or corrupt')
    intent = record['payload']
    if intent['identity'] != _identity(payload) or intent['repositoryRoot'].casefold() != str(root.absolute()).casefold():
        raise JenkinsError('patch_identity_mismatch', 'Patch reference belongs to another request or checkout')
    if payload.get('buildRoot') and _build_root(root, payload['buildRoot']) != intent['buildRoot']:
        raise JenkinsError('patch_build_root_mismatch', 'Patch preparation must retain its registered build root')
    _authorize(state, intent)
    return intent


def _verify_sources(root, intent, *, after=False):
    if _head(root) != intent['baseHead']:
        raise JenkinsError('patch_base_changed', 'Git HEAD changed after patch registration')
    for path, checksum in intent['dependencyHashes'].items():
        canonical_path(root, path)
        if _before_hash(root, path) != checksum:
            raise JenkinsError('patch_dependency_changed', 'Declared dependency changed after patch registration', details={'path': path})
    if after:
        for path, text in intent['renderedPatch'].items():
            canonical_path(root, path)
            checksum = None if text is None else hashlib.sha256(text.encode('utf-8')).hexdigest()
            if _before_hash(root, path) != checksum:
                raise JenkinsError('foreign_edit_detected', 'Applied patch differs from the registered postimage', details={'path': path})


def _prepared_request(intent, result, ref):
    identity = result['identity']
    return {**identity, 'sourceInputDigest': result['sealedInputRef'], 'sealedInputRef': result['sealedInputRef'],
            'coverageDigest': identity['coverageDigest'], 'patchRequestRef': ref,
            'patchOperationRef': result['patchOperationRef'], 'ownedPaths': sorted(intent['beforeHashes']),
            'buildRoot': intent['buildRoot'], 'driverDigest': identity['stageImplementationDigest'],
            'authorizationSnapshot': intent['authorizationSnapshot']}


def prepare_patch(state: State, repo_root: Path, payload: dict, *, trusted_driver_digest: str) -> dict:
    root = repo_root.absolute()
    intent = _load_intent(state, root, payload)
    identity = intent['identity']
    if identity['stageImplementationDigest'] != trusted_driver_digest:
        raise JenkinsError('driver_context_mismatch', 'Patch preparation must use its registered sealed driver')
    ref = payload['patchRequestRef']
    with _lock(root):
        prior = state.get('patch_preparation', ref)
        if prior:
            if prior['payload']['status'] != 'prepared':
                raise JenkinsError('patch_recovery_required', 'Patch preparation failed; reconcile before a new request')
            result = prior['payload']['result']
            sealed = state.get('sealed_input', result['sealedInputRef'])
            operation = state.get_operation(result['patchOperationRef'])
            if not sealed or sealed['payload'].get('status') != 'sealed' or not operation or operation['status'] != 'complete':
                raise JenkinsError('patch_binding_missing', 'Prepared patch no longer has its sealed input and complete journal')
            state.submit_request(_prepared_request(intent, result, ref))
            return result
        operation_id = 'patch-' + ref
        request = state.get_request(identity['repositoryId'], identity['sessionId'], identity['requestId'])
        if request and request['payload'].get('patchRequestRef') != ref:
            raise JenkinsError('patch_request_conflict', 'Request identity changed before patch application')
        operation = state.get_operation(operation_id)
        if operation and operation['status'] == 'aborted':
            raise JenkinsError('patch_recovery_required', 'Aborted source patch cannot be sealed or applied again')
        try:
            _verify_sources(root, intent)
            if digest(intent['renderedPatch']) != intent['renderedDigest']:
                raise JenkinsError('patch_intent_corrupt', 'Registered patch postimage is corrupt')
            if operation is None:
                rendered = materialize_patch(root, intent['unifiedPatch'], intent['beforeHashes'])
                if rendered != intent['renderedPatch']:
                    raise JenkinsError('patch_intent_corrupt', 'Patch parser output differs from the registered postimage')
            authorization = {**identity, 'owner': identity['attemptId']}
            operation = apply_owned_patch(state, root, identity['attemptId'], intent['renderedPatch'],
                                           intent['beforeHashes'], operation_id=operation_id,
                                           authorization=authorization, build_root=intent['buildRoot'])
            if operation['status'] != 'complete':
                raise JenkinsError('patch_not_complete', 'Patch must be completely applied before sealing')
            _verify_sources(root, intent, after=True)
            claims = claim_paths(state, root, identity['attemptId'], intent['beforeHashes'])
            sealed = source_handle('seal', {**authorization, 'claims': [claim.__dict__ for claim in claims],
                'coverage': intent['coverage'], 'baseHead': intent['baseHead'],
                'declaredDependencies': intent['declaredDependencies'], 'externalInputs': intent['externalInputs'],
                'patchOperationRefs': [operation_id], 'buildRoot': intent['buildRoot'],
                'objectRoot': str(canonical_build_root(intent['buildRoot']).path / 'zircon-jenkins')}, state, root)
            _verify_sources(root, intent, after=True)
            record = state.get('sealed_input', sealed['sourceDigest'])
            final_identity = {**identity, 'sourceInputDigest': sealed['sourceDigest'],
                              'coverageDigest': record['payload']['coverageDigest']}
            result = response('prepared', identity=final_identity, sealedInputRef=sealed['sourceDigest'],
                              patchOperationRef=operation_id, patchRequestRef=ref, buildRoot=intent['buildRoot'])
            # Save the binding before State.submit_request: recovery reuses this
            # identity even if external dependencies change after sealing.
            state.put('patch_preparation', ref, {'status': 'prepared', 'result': result}, expected_version=0)
            state.submit_request(_prepared_request(intent, result, ref))
            return result
        except BaseException as error:
            if state.get('patch_preparation', ref) is None:
                state.put('patch_preparation', ref, {'status': 'failed', 'operationId': operation_id,
                          'reasonCode': getattr(error, 'code', type(error).__name__)}, expected_version=0)
            raise


def reconcile_patch(state: State, repo_root: Path, payload: dict, *, trusted_driver_digest=None) -> dict:
    from .patch_recovery import reconcile_preparation
    root = repo_root.absolute()
    intent = _load_intent(state, root, payload)
    with _lock(root):
        return reconcile_preparation(state, root, intent, payload['patchRequestRef'])


def submit_patch(state: State, repo_root: Path, payload: dict, *, transport=None, manager=None) -> dict:
    from .submission import dispatch_registered, DEFAULT_BASE_URL
    root = repo_root.absolute()
    intent = _load_intent(state, root, payload)
    identity = intent['identity']
    preparation = state.get('patch_preparation', payload['patchRequestRef'])
    if preparation and preparation['payload'].get('status') != 'prepared':
        raise JenkinsError('patch_recovery_required', 'Failed patch request cannot be dispatched again')
    request = state.get_request(identity['repositoryId'], identity['sessionId'], identity['requestId'])
    if request and request['payload'].get('patchRequestRef') != payload['patchRequestRef']:
        raise JenkinsError('patch_request_conflict', 'Registered request belongs to a different source input')
    parameters = {'patchRequestRef': payload['patchRequestRef'], 'attemptId': identity['attemptId'],
                  'generation': str(identity['generation']), 'buildRoot': intent['buildRoot'],
                  'stageImplementationDigest': identity['stageImplementationDigest']}
    dispatch = {key: identity[key] for key in _IDENTITY[:3]}
    dispatch['parameters'] = parameters
    fields = {'REPOSITORY_ID': identity['repositoryId'], 'SESSION_ID': identity['sessionId'],
              'REQUEST_ID': identity['requestId'], 'ATTEMPT_ID': identity['attemptId'], 'GENERATION': '1',
              'PATCH_REQUEST_REF': payload['patchRequestRef'], 'BUILD_ROOT': intent['buildRoot'],
              'STAGE_IMPLEMENTATION_DIGEST': identity['stageImplementationDigest']}
    if manager is None and transport is None:
        from .deployment.spec import load_spec
        from .deployment.paths import resolve_paths
        from .deployment.manager import DeploymentManager
        spec = load_spec(root / '.jenkins/deployment-spec.json')
        manager = DeploymentManager(spec, resolve_paths(spec))
    return dispatch_registered(state, dispatch, fields, registered_request=state.get('patch_intent', payload['patchRequestRef']),
                               base_url=manager.base_url if manager else DEFAULT_BASE_URL,
                               job='zircon-flow', transport=transport, manager=manager)


def handle(action, payload, state, repo_root, *, domain='patch'):
    if action == 'register':
        from .workflow.handler import _trusted_driver
        return register_patch_intent(state, repo_root, payload,
                                     trusted_driver_digest=_trusted_driver(repo_root, _identity(payload)))
    if action == 'submit':
        return submit_patch(state, repo_root, payload)
    if action in {'reconcile', 'cancel'}:
        return reconcile_patch(state, repo_root, payload)
    if action == 'query':
        intent = _load_intent(state, repo_root, payload)
        preparation = state.get('patch_preparation', payload['patchRequestRef'])
        return response(preparation['payload']['status'] if preparation else 'registered',
                        patchRequestRef=payload['patchRequestRef'], preparation=preparation,
                        request=state.get_request(*[intent['identity'][key] for key in _IDENTITY[:3]]))
    raise JenkinsError('operation_unknown', 'Unknown patch intake operation')
