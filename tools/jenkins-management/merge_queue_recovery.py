"""Pure merge-queue decisions; the Jenkins worker retains all mutation authority."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


class QueueBlocked(Exception):
    def __init__(self, status, reason, prerequisites=()):
        super().__init__(reason)
        self.status = status
        self.prerequisites = list(prerequisites)


def eligible(record):
    return (record.get('status') == 'committed' and bool(record.get('sourceCommit'))
            and record.get('integrationStatus') not in {'complete', 'awaiting-main-sync'})


def select_operations(records, limit=8, active_owner=None):
    """Least recently attempted first; reorder selected dependencies without starvation."""
    if type(limit) is not int or limit < 1:
        raise ValueError('A positive bounded batch size is required')
    pending = {op: rec for op, rec in records.items() if eligible(rec)}
    def rank(op):
        rec = pending[op]
        return (rec.get('periodicIntegration', {}).get('build', 0), rec.get('committedAt', 0), op)
    ordered = sorted(pending, key=rank)
    if active_owner in pending:
        ordered.remove(active_owner)
        ordered.insert(0, active_owner)
    chosen = ordered[:limit]
    selected, visiting, result = set(chosen), set(), []
    def visit(op):
        if op in result or op in visiting:
            return
        visiting.add(op)
        for dependency in pending[op].get('periodicIntegration', {}).get('prerequisiteOperations', []):
            if dependency in selected:
                visit(dependency)
        visiting.remove(op)
        result.append(op)
    for op in chosen:
        visit(op)
    return result


def _scoped(paths, allowed):
    folded = [name.casefold() for name in paths]
    return len(set(folded)) == len(folded) and set(folded) <= {name.casefold() for name in allowed}


def require_scoped_history(record, authored_paths, own_paths, prerequisite_operations):
    """A scoped last commit cannot authorize its undeclared predecessor history."""
    if not _scoped(own_paths, record['request']['paths']):
        raise QueueBlocked('scope-blocked', 'The original task commit itself contains undeclared paths; owner correction required')
    if _scoped(authored_paths, record['request']['paths']):
        return
    if prerequisite_operations:
        raise QueueBlocked('waiting-prerequisite', 'Known predecessor commits must enter main before this scoped successor', sorted(set(prerequisite_operations)))
    raise QueueBlocked('scope-blocked', 'Unmerged history contains undeclared paths with no registered predecessor; retain source for its owner')


def _checked_ref(ref, read_bytes):
    raw = read_bytes(Path(ref['path']))
    if len(raw) != ref['bytes'] or hashlib.sha256(raw).hexdigest() != ref['sha256']:
        raise ValueError('Original native evidence changed: ' + str(ref['path']))
    return raw


def prove_candidate_terminal(record, marker, facts, read_bytes, native_complete):
    """Reject unknown ownership, work and outcomes before resuming/releasing a candidate."""
    def require(condition, reason):
        if not condition:
            raise ValueError(reason)
    operation = record['request']['operationId']
    root = Path(facts['candidateRoot'])
    require(marker.get('status') == 'active' and marker.get('operationId') == operation, 'Candidate owner changed')
    require(Path(marker['path']) == root == Path(record['candidateRoot']), 'Candidate path changed')
    require(facts['commonDirectory'] == facts['repositoryGitDirectory'], 'Candidate repository identity changed')
    require(not facts['indexLocked'] and not facts['unfinishedOperations'] and not facts['dirty'], 'Active or modified Git candidate retained')
    require(facts['head'] == record.get('candidateHead'), 'Candidate HEAD changed')
    require(marker.get('mainHead') == record.get('mainHeadBefore'), 'Original candidate base changed')
    info = record.get('candidateGit', {})
    require(info.get('nativeComplete') is True and info.get('operationKind') == 'merge' and info.get('rootExitCode') == 0, 'Original successful merge terminal missing')
    command = json.loads(_checked_ref(info['commandRef'], read_bytes))
    native = json.loads(_checked_ref(info['nativeTerminalRef'], read_bytes))
    started = json.loads(_checked_ref(info['startedRef'], read_bytes))
    _checked_ref(info['stdoutRef'], read_bytes)
    _checked_ref(info['stderrRef'], read_bytes)
    require(command['operationId'] == operation and command['operationKind'] == 'merge', 'Original native operation changed')
    require(Path(command['candidateRoot']) == root == Path(command['cwd']), 'Original native checkout changed')
    require(command['command'][-4:] == ['merge', '--no-edit', '--no-gpg-sign', record['sourceCommit']], 'Original native source changed')
    executable = command['executable']
    require(command['command'][0] == executable['path'] and Path(executable['path']).name.casefold() == 'git.exe', 'Original Git executable changed')
    _checked_ref(executable, read_bytes)
    require(native_complete(native, command['command'], root) and native['rootExitCode'] == 0 and not native.get('cleanup', {}).get('attempted'), 'Original Git job has no natural terminal')
    require(all(started.get(key) == native.get(key) for key in ('pid', 'creationTime', 'command', 'cwd')), 'Original process birth identity changed')
    validation = record.get('candidateValidation', {})
    require(not validation.get('preparationPending'), 'Original validation preparation remains pending')
    require(validation.get('status') in {'passed', 'failed', 'skipped'}, 'Original validation outcome remains unknown')
    require(validation.get('head') == facts['head'] and validation.get('runtime') == command['runtime'], 'Original validation identity changed')
    require(isinstance(validation.get('steps'), list), 'Original validation steps missing')
    for step in validation['steps']:
        require(step.get('complete') is True and Path(step['cwd']) == root
                and native_complete(step.get('native'), step['command'], root), 'Original validation job has no terminal')
    return {'operationId': operation, 'sourceCommit': record['sourceCommit'], 'candidateHead': facts['head'],
            'mainHead': record['mainHeadBefore'], 'runtime': command['runtime'], 'nativeEvidenceVerified': True,
            'validationStatus': validation['status'], 'originalRequestUnchanged': True}


def validation_request(request):
    """Keep compiler acceptance separate from the bounded queue maintenance job."""
    selected = dict(request)
    recipes = request.get('validation', [])
    selected['validation'] = [recipe for recipe in recipes if recipe.get('kind') != 'cargo']
    deferred = [recipe for recipe in recipes if recipe.get('kind') == 'cargo']
    return selected, deferred
