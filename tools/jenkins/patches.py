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

from .contracts import JenkinsError, digest, file_digest, identifier, response
from .resources import canonical_build_root
from .source import (SealedManifest, _before_hash, _thaw, apply_owned_patch, canonical_path,
                     claim_paths, collect_cargo_evidence, get_sealed_manifest,
                     handle as source_handle, persist_cargo_evidence,
                     repository_id)
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


def _policy_tool(policy: Mapping, name: str, digest_name: str,
                 executable_names: set[str]) -> str:
    """Resolve one executable from the verified environment policy.

    The patch request never supplies tool paths or metadata. A policy is
    accepted only when it names an absolute executable with the expected
    content digest; this keeps metadata collection on the same toolchain that
    the recipe planner will later bind.
    """
    value = policy.get(name)
    if not isinstance(value, str) or not value:
        raise JenkinsError("environment_policy_incomplete",
                           f"trusted environment is missing {name}")
    path = Path(value)
    if not path.is_absolute() or path.name.casefold() not in executable_names:
        raise JenkinsError("environment_policy_tool_invalid",
                           f"trusted environment {name} is not an absolute executable")
    path = path.absolute()
    if not path.is_file():
        raise JenkinsError("environment_policy_tool_missing",
                           f"trusted environment {name} is unavailable")
    expected = policy.get(digest_name)
    if (not isinstance(expected, str) or len(expected) != 64
            or any(char not in "0123456789abcdef" for char in expected)
            or file_digest(path) != expected):
        raise JenkinsError("environment_policy_tool_changed",
                           f"trusted environment {name} digest differs")
    return str(path)


def _verified_metadata_inputs(state: State, policy_root: Path) -> tuple[dict, dict, str, str]:
    """Load the deployment-owned environment and its exact Cargo tools."""
    from .workflow.planning import _environment_policy

    environment, policy, _ = _environment_policy(state, policy_root)
    cargo = _policy_tool(policy, "cargoPath", "cargoSha256", {"cargo", "cargo.exe"})
    rustc = _policy_tool(policy, "rustcPath", "rustcSha256", {"rustc", "rustc.exe"})
    return environment, policy, cargo, rustc


def _sealed_metadata_binding(state: State, root: Path, intent: Mapping,
                            sealed_result: Mapping) -> tuple[object, dict, str, str]:
    """Load the sealed manifest and verify its consumer/build-root identity."""
    identity = intent["identity"]
    source_digest = sealed_result.get("sourceDigest")
    if not isinstance(source_digest, str) or not source_digest:
        raise JenkinsError("sealed_input_missing", "sealed patch preparation has no source digest")
    manifest = get_sealed_manifest(
        state, repository_id_value=identity["repositoryId"],
        session_id=identity["sessionId"], owner=identity["attemptId"],
        source_digest=source_digest)
    record = state.get("sealed_input", source_digest)
    sealed = record["payload"] if record else None
    if not isinstance(sealed, Mapping) or sealed.get("status") != "sealed":
        raise JenkinsError("sealed_input_missing", "sealed patch input is unavailable")
    # The same sealed manifest may have multiple immutable consumers. The
    # current session/owner is checked by get_sealed_manifest above; the shared
    # sealed record itself only needs to retain the repository binding.
    if sealed.get("repositoryId") != identity["repositoryId"]:
        raise JenkinsError("sealed_identity_mismatch",
                           "sealed patch input is bound to another repository")
    sealed_root_value = sealed.get("repositoryRoot")
    if not isinstance(sealed_root_value, str) or not sealed_root_value:
        raise JenkinsError("sealed_repository_missing",
                           "sealed patch input has no source checkout binding")
    sealed_root = Path(sealed_root_value).absolute()
    # The sealed source checkout is a physical coordinator-owned root. Reject
    # lexical aliases and reparse points before comparing it with the resolver's
    # selected checkout; policy_root remains a separate deployment context.
    from .deployment.driver import _plain
    _plain(root)
    _plain(sealed_root)
    if (str(sealed_root).casefold() != str(root).casefold()
            or sealed_root.resolve() != sealed_root
            or root.resolve() != root):
        raise JenkinsError("sealed_repository_mismatch",
                           "sealed patch input belongs to another physical checkout")
    expected_build = str(canonical_build_root(intent["buildRoot"]).path)
    sealed_build = sealed.get("buildRoot")
    if (not isinstance(sealed_build, str)
            or str(canonical_build_root(sealed_build).path) != expected_build):
        raise JenkinsError("patch_build_root_mismatch",
                           "sealed patch input changed its registered build root")
    expected_objects = str(canonical_build_root(intent["buildRoot"]).namespace())
    if sealed.get("objectRoot") != expected_objects:
        raise JenkinsError("object_root_mismatch",
                           "sealed patch input uses an unapproved object root")
    return manifest, dict(sealed), expected_build, expected_objects


def _validate_metadata_workspace(evidence: Mapping, build_root: str, source_digest: str, *, manifest: Mapping | SealedManifest | None = None) -> None:
    from .resources import physical_path_under
    physical_root = canonical_build_root(build_root)
    expected = physical_path_under(physical_root, physical_root.namespace() / "inputs" / "runs" / source_digest)
    if manifest:
        from .source.external_inputs import sources
        if isinstance(manifest, SealedManifest):
            manifest = _thaw(manifest.payload)
        if sources(manifest):
            expected = physical_path_under(physical_root, expected / "source")
    paths = [evidence[key] for key in ("materializedRoot", "workspaceRoot", "workspace_root", "runRoot")
             if key in evidence]
    if not paths or any(not isinstance(value, str) or not value for value in paths):
        raise JenkinsError("cargo_workspace_missing", "Cargo evidence has no sealed workspace identity")
    for value in paths:
        if physical_path_under(physical_root, value) != expected:
            raise JenkinsError("cargo_workspace_identity_mismatch", "Cargo evidence workspace differs from its sealed input")


def _validate_existing_metadata_evidence(state: State, record: Mapping, *, identity: Mapping,
                                         source_digest: str, build_root: str,
                                         policy_digest: str) -> Mapping:
    evidence = record.get("payload") if isinstance(record, Mapping) else None
    if not isinstance(evidence, Mapping):
        raise JenkinsError("cargo_evidence_corrupt", "sealed Cargo evidence is unavailable")
    if (evidence.get("sourceDigest") != source_digest
            or evidence.get("manifestDigest", source_digest) != source_digest
            or evidence.get("repositoryId") != identity["repositoryId"]
            or evidence.get("buildRoot") != build_root
            or evidence.get("environmentPolicyDigest") != policy_digest):
        raise JenkinsError("cargo_evidence_conflict",
                           "sealed Cargo evidence is bound to another request or environment")
    # Reuse must satisfy the same workspace and metadata path checks as a fresh
    # collector result. This rejects a stale path or an evidence record that
    # points outside the approved physical build root.
    sealed = state.get("sealed_input", source_digest)
    manifest = sealed["payload"].get("sourceManifest") or sealed["payload"].get("manifest") if sealed else None
    _validate_metadata_workspace(evidence, build_root, source_digest, manifest=manifest)
    from .workflow.planning import _cargo_evidence
    _cargo_evidence(state, source_digest)
    return evidence


def _ensure_patch_metadata(state: State, root: Path, intent: Mapping,
                           sealed_result: Mapping, *, policy_root: Path | None = None) -> None:
    """Collect and persist authoritative Cargo metadata for a sealed patch.

    Formal deployments must have a verified environment policy. Small offline
    unit fixtures intentionally omit deployment state and policy, so they keep
    exercising patch sealing without launching Cargo. No caller field is read
    for tool paths, environment values, metadata, or evidence identity.
    """
    context_root = (policy_root or root).absolute()
    marker = context_root / ".jenkins/state/deployment/driver.json"
    policy_record = state.get("environment_policy", "default")
    if policy_record is None and not marker.is_file():
        return
    manifest, sealed, expected_build, object_root = _sealed_metadata_binding(
        state, root, intent, sealed_result)
    environment, policy, cargo, rustc = _verified_metadata_inputs(
        state, context_root)
    # Bind evidence to the complete policy record, not only its environment
    # variable subset. The planner may derive execution variables separately.
    policy_digest = digest(policy)
    source_digest = str(sealed_result["sourceDigest"])
    existing = state.get("sealed_cargo_evidence", source_digest)
    if existing is not None:
        _validate_existing_metadata_evidence(
            state, existing, identity=intent["identity"], source_digest=source_digest,
            build_root=expected_build, policy_digest=policy_digest)
        return
    evidence = collect_cargo_evidence(
        manifest, object_root, expected_build, cargo=cargo, rustc=rustc,
        environment=environment, environment_policy_digest=policy_digest)
    if (not isinstance(evidence, Mapping)
            or evidence.get("sourceDigest") != source_digest
            or evidence.get("manifestDigest", source_digest) != source_digest
            or evidence.get("buildRoot") != expected_build
            or evidence.get("environmentPolicyDigest") != policy_digest):
        raise JenkinsError("cargo_evidence_identity_mismatch",
                           "collected Cargo evidence is not bound to the sealed patch")
    _validate_metadata_workspace(evidence, expected_build, source_digest, manifest=manifest)
    persist_cargo_evidence(
        state, evidence, repository_id_value=intent["identity"]["repositoryId"],
        session_id=intent["identity"]["sessionId"],
        owner=intent["identity"]["attemptId"], manifest=manifest)
    from .workflow.planning import _cargo_evidence
    _cargo_evidence(state, source_digest)


def _runtime_binding(root: Path, identity: Mapping, supplied: Mapping | None = None) -> str | None:
    """Resolve the Jenkins runtime operation fence for a patch dispatch.

    A deployed checkout must derive this value from the selected sealed driver;
    accepting a caller-only value would let a patch enter a different runtime.
    Isolated fixtures without a deployment selector may provide the value
    explicitly so their transport tests remain bounded and offline.
    """
    supplied_value = None
    for source in (supplied or {}, identity):
        value = source.get("runtimeOperationId")
        if value not in (None, ""):
            supplied_value = str(value)
            break
    marker = root / ".jenkins/state/deployment/driver.json"
    if marker.exists():
        from .deployment.driver import active_driver_binding
        try:
            binding = active_driver_binding(root)
        except Exception as error:
            if isinstance(error, JenkinsError):
                raise
            raise JenkinsError("active_driver_unavailable", "The active driver is unavailable") from error
        runtime = str(binding.get("runtimeOperationId") or "")
        if not runtime:
            raise JenkinsError("runtime_binding_missing", "The selected driver has no runtime operation binding")
        driver_digest = str(binding.get("digest") or "")
        if driver_digest != str(identity.get("stageImplementationDigest") or ""):
            raise JenkinsError("driver_context_mismatch", "Patch identity differs from the selected driver")
        if supplied_value is not None and supplied_value != runtime:
            raise JenkinsError("runtime_binding_mismatch", "Patch runtime operation differs from the selected driver")
        return runtime
    return supplied_value


def _head(root: Path) -> str | None:
    if not (root / '.git').exists():
        return None  # A bounded source fixture has no Git database.
    completed = subprocess.run(['git', '-C', str(root), 'rev-parse', '--verify', 'HEAD'],
                               capture_output=True, text=True)
    if completed.returncode:
        raise JenkinsError('git_base_unavailable', 'Patch intake cannot read the checkout HEAD')
    return completed.stdout.strip()


def _registered_runtime(root: Path, intent: Mapping, payload: Mapping) -> str | None:
    """Retain a new intent's runtime without upgrading historical intent."""
    registered = intent.get('runtimeOperationId')
    supplied = dict(payload)
    if registered is not None and 'runtimeOperationId' not in supplied:
        supplied['runtimeOperationId'] = registered
    runtime = _runtime_binding(root, intent['identity'], supplied)
    if registered is None:
        if (root / '.jenkins/state/deployment/driver.json').exists():
            raise JenkinsError('patch_runtime_binding_missing', 'Historical patch intent has no immutable runtime binding')
    elif runtime != registered:
        raise JenkinsError('patch_runtime_binding_mismatch', 'Patch control must retain its registered runtime operation')
    return runtime


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
    mode = payload.get('inputMode', 'unapplied')
    if mode == 'already-applied':
        from .already_applied import register_already_applied_intent
        return register_already_applied_intent(state, root, payload,
                                              trusted_driver_digest=trusted_driver_digest)
    if mode != 'unapplied':
        raise JenkinsError('patch_input_mode_invalid', 'Unknown incremental patch input mode')
    if any(key in payload for key in ('afterHashes', 'dependencyHashes', 'baselineRef')):
        raise JenkinsError('patch_input_mode_conflict', 'Applied-input evidence requires already-applied mode')
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
    runtime = _runtime_binding(root, identity, payload)
    if runtime is not None:
        supplied['runtimeOperationId'] = runtime
    input_digest = digest(supplied)
    slot = digest([identity[key] for key in _IDENTITY[:3]])
    owner_slot = digest([repository_id(root), identity['attemptId']])
    with _lock(root):
        prior = state.get('patch_request', slot)
        if prior:
            intent = _load_intent(state, root, {'identity': identity, 'patchRequestRef': prior['payload']['patchRequestRef']})
            replay = dict(supplied)
            if 'runtimeOperationId' not in intent:
                replay.pop('runtimeOperationId', None)
            else:
                _registered_runtime(root, intent, payload)
            if prior['payload'].get('inputDigest') != digest(replay):
                raise JenkinsError('patch_request_conflict', 'This request already owns a different patch intent')
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
    if intent['identity'] != _identity(payload):
        raise JenkinsError('patch_identity_mismatch', 'Patch reference belongs to another request')
    from .patch_context import resolve_patch_checkout
    resolve_patch_checkout(state, root, intent)
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
    request = {**identity, 'sourceInputDigest': result['sealedInputRef'], 'sealedInputRef': result['sealedInputRef'],
            'coverageDigest': identity['coverageDigest'], 'patchRequestRef': ref,
            'patchOperationRef': result['patchOperationRef'], 'ownedPaths': sorted(intent['beforeHashes']),
            'buildRoot': intent['buildRoot'], 'driverDigest': identity['stageImplementationDigest'],
            'authorizationSnapshot': intent['authorizationSnapshot']}
    if intent.get('runtimeOperationId') is not None:
        request['runtimeOperationId'] = intent['runtimeOperationId']
    return request


def prepare_patch(state: State, repo_root: Path, payload: dict, *, trusted_driver_digest: str,
                  policy_root: Path | None = None) -> dict:
    context_root = repo_root.absolute()
    intent = _load_intent(state, context_root, payload)
    if intent.get('inputMode') == 'already-applied':
        from .already_applied import prepare_already_applied
        return prepare_already_applied(state, context_root, payload,
                                       trusted_driver_digest=trusted_driver_digest,
                                       policy_root=policy_root or context_root)
    root = Path(intent['repositoryRoot']).absolute()
    policy_root = (policy_root or context_root).absolute()
    identity = intent['identity']
    if identity['stageImplementationDigest'] != trusted_driver_digest:
        raise JenkinsError('driver_context_mismatch', 'Patch preparation must use its registered sealed driver')
    _registered_runtime(context_root, intent, payload)
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
            _ensure_patch_metadata(state, root, intent, sealed['payload'], policy_root=policy_root)
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
                'objectRoot': str(canonical_build_root(intent['buildRoot']).namespace())}, state, root)
            _verify_sources(root, intent, after=True)
            _ensure_patch_metadata(state, root, intent, sealed, policy_root=policy_root)
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
    intent = _load_intent(state, repo_root.absolute(), payload)
    if intent.get('inputMode') == 'already-applied':
        from .already_applied import reconcile_already_applied
        return reconcile_already_applied(state, repo_root.absolute(), payload,
                                         trusted_driver_digest=trusted_driver_digest)
    root = Path(intent['repositoryRoot']).absolute()
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
    runtime_operation_id = (_runtime_binding(root, identity, payload) if intent.get('inputMode') == 'already-applied'
                            else _registered_runtime(root, intent, payload))
    parameters = {'patchRequestRef': payload['patchRequestRef'], 'attemptId': identity['attemptId'],
                  'generation': str(identity['generation']), 'buildRoot': intent['buildRoot'],
                  'stageImplementationDigest': identity['stageImplementationDigest']}
    if runtime_operation_id is not None:
        parameters['runtimeOperationId'] = runtime_operation_id
    dispatch = {key: identity[key] for key in _IDENTITY[:3]}
    dispatch['parameters'] = parameters
    fields = {'REPOSITORY_ID': identity['repositoryId'], 'SESSION_ID': identity['sessionId'],
              'REQUEST_ID': identity['requestId'], 'ATTEMPT_ID': identity['attemptId'], 'GENERATION': '1',
              'PATCH_REQUEST_REF': payload['patchRequestRef'], 'BUILD_ROOT': intent['buildRoot'],
              'STAGE_IMPLEMENTATION_DIGEST': identity['stageImplementationDigest']}
    if runtime_operation_id is not None:
        fields['RUNTIME_OPERATION_ID'] = runtime_operation_id
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
        from .patch_context import patch_checkout_for_registration
        context_root = repo_root.absolute()
        trusted = _trusted_driver(context_root, _identity(payload))
        registered_payload = {**payload, 'buildRoot': _build_root(context_root, payload.get('buildRoot'))}
        if payload.get('inputMode') == 'already-applied':
            return register_patch_intent(state, context_root, registered_payload, trusted_driver_digest=trusted)
        runtime = _runtime_binding(context_root, _identity(payload), payload)
        if runtime is not None:
            registered_payload['runtimeOperationId'] = runtime
        checkout = patch_checkout_for_registration(state, context_root, payload)
        return register_patch_intent(state, checkout, registered_payload, trusted_driver_digest=trusted)
    if action == 'submit':
        return submit_patch(state, repo_root, payload)
    if action in {'reconcile', 'cancel'}:
        if action == 'cancel':
            intent = _load_intent(state, repo_root.absolute(), payload)
            if intent.get('inputMode') == 'already-applied':
                from .already_applied import reconcile_already_applied
                return reconcile_already_applied(state, repo_root.absolute(), payload, cancel=True)
        return reconcile_patch(state, repo_root, payload)
    if action == 'query':
        intent = _load_intent(state, repo_root, payload)
        preparation = state.get('patch_preparation', payload['patchRequestRef'])
        return response(preparation['payload']['status'] if preparation else 'registered',
                        patchRequestRef=payload['patchRequestRef'], preparation=preparation,
                        request=state.get_request(*[intent['identity'][key] for key in _IDENTITY[:3]]))
    raise JenkinsError('operation_unknown', 'Unknown patch intake operation')
