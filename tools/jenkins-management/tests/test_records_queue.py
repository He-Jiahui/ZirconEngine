"""Focused intake contract tests; no compiler, source snapshot, or inbox writes."""
import importlib.util
from pathlib import Path
import unittest
from contextlib import ExitStack, nullcontext
import copy
import json
from unittest.mock import patch, Mock

source = Path(__file__).absolute().parents[1] / 'records_queue.py'
spec = importlib.util.spec_from_file_location('records_queue_tests_subject', source)
queue = importlib.util.module_from_spec(spec)
spec.loader.exec_module(queue)


class IntakeTests(unittest.TestCase):
    def inventory(self):
        return {'schemaVersion': 1,
                'authorization': 'user: every 30 minutes; enqueue all tasks, archive evidence, then empty inbox',
                'tasks': [{'taskId': 'one', 'kind': 'task', 'originalIdentity': {'sessionId': 'original'},
                           'pendingRequirements': ['source-ready'], 'messages': [
                               {'path': 'E:/Git/ZirconEngine/.jenkins/records/one.md', 'sha256': '1' * 64}]}]}

    def test_rejects_duplicate_message_in_different_task(self):
        value = self.inventory()
        value['tasks'].append({**value['tasks'][0], 'taskId': 'two'})
        with self.assertRaisesRegex(ValueError, 'unique exact'):
            queue.validate_inventory('E:/Git/ZirconEngine', value)

    def test_rejects_path_traversal_and_outside_inbox(self):
        for path in ('E:/Git/ZirconEngine/.jenkins/records/../state/one.md',
                     'E:/Git/ZirconEngine/.jenkins/state/one.md'):
            value = self.inventory()
            value['tasks'][0]['messages'][0]['path'] = path
            with self.assertRaises(ValueError):
                queue.validate_inventory('E:/Git/ZirconEngine', value)

    def test_rejects_execution_with_other_session_or_external_action(self):
        for payload in ({'action': 'commit-worktree', 'sessionId': 'original'},
                        {'action': 'validate', 'sessionId': 'borrowed'}):
            value = self.inventory()
            value['tasks'][0]['formalRequests'] = [payload]
            with self.assertRaises(ValueError):
                queue.validate_inventory('E:/Git/ZirconEngine', value)

    def test_changed_inbox_is_never_archived_or_removed(self):
        message = {'path': 'E:/Git/ZirconEngine/.jenkins/records/one.md', 'bytes': 7,
                   'mtimeNs': 1, 'sha256': '1' * 64}
        with patch.object(queue, 'fingerprint', return_value=({**message, 'sha256': '2' * 64}, b'changed')):
            with patch.object(queue, 'write_once') as write:
                # Use an existing file only as a stand-in for the existence check.
                with patch.object(Path, 'exists', return_value=True), patch.object(queue, 'plain', side_effect=Path):
                    with self.assertRaisesRegex(ValueError, 'changed'):
                        queue.archive_messages(Path('E:/Git/ZirconEngine'), Path('E:/Git/ZirconEngine/.jenkins/state/records-queue'),
                                               {'messages': [message]}, '1' * 64)
                write.assert_not_called()

    def test_lost_acknowledgement_never_causes_second_submission(self):
        for lose_ack in (False, True):
            with self.subTest(lose_ack=lose_ack), RecordsTransportCase(
                    queue, self.inventory(), lose_ack=lose_ack) as case:
                first = queue.submit(case.repo, case.inventory)
                self.assertEqual(queue.sys.path[0], str(case.driver_root))
                second = queue.submit(case.repo, case.inventory)
                self.assertEqual(first['receipts'][0]['phase'], 'delivery_unknown' if lose_ack else 'queued')
                self.assertEqual(first, second)
                self.assertEqual(len(case.posts), 1)
                endpoint, parameters = case.posts[0]
                self.assertEqual(endpoint, '/job/zircon-records/buildWithParameters')
                self.assertEqual(parameters, {'INVENTORY_DIGEST': [first['inventoryDigest']],
                                              'TASK_ID': ['one'], 'MODE': ['task']})
                case.api.record.return_value = {'payload': {'taskId': 'one', 'pendingRequirements': []}}
                with patch('builtins.print') as output:
                    self.assertEqual(queue.main(['query', '--repo-root', str(case.repo), '--task-id', 'one']), 0)
                self.assertEqual(json.loads(output.call_args.args[0])['payload']['taskId'], 'one')
                self.assertEqual(len(case.posts), 1)
                case.api.capability.assert_not_called()
                old = case.raw[str(case.driver_member)]
                case.raw[str(case.driver_member)] = b'changed'
                with self.assertRaisesRegex(ValueError, 'Pinned driver'):
                    queue.binding(case.repo, transport_only=True)
                case.raw[str(case.driver_member)] = old
                self.assertEqual(len(case.posts), 1)
                with self.assertRaisesRegex(PermissionError, 'OpenProcess'):
                    queue.work(case.repo, 'one', first['inventoryDigest'], case.repo / 'unused-output.json')
                case.api.capability.assert_called_once_with(case.repo)

    def cleanup_case(self, deleted=True):
        root = Path('E:/Git/ZirconEngine/.jenkins/state/records-queue')
        message = {'path': 'E:\\Git\\ZirconEngine\\.jenkins\\records\\one.md', 'bytes': 7,
                   'sha256': '1' * 64, 'archivePath': str(root / 'archive/one.md')}
        receipt = {'success': False, 'taskId': 'one', 'inventoryDigest': '2' * 64,
                   'deletedFiles': [{k: message[k] for k in ('path', 'bytes', 'sha256')}] if deleted else [],
                   'error': 'receipt accounting failure'}
        return root, message, receipt

    def test_partial_native_deletion_cannot_be_marked_complete(self):
        root, message, receipt = self.cleanup_case(deleted=False)
        with patch.object(queue, 'plain', side_effect=Path), \
             patch.object(Path, 'exists', return_value=True), \
             patch.object(queue, 'read_json', return_value=(receipt, {})), \
             patch.object(queue, 'write_once') as write:
            with self.assertRaisesRegex(ValueError, 'incomplete'):
                queue.cleanup({}, {}, root, 'one', '2' * 64, [message])
            write.assert_not_called()

    def test_missing_archive_blocks_receipt_reconciliation(self):
        root, message, receipt = self.cleanup_case()
        with patch.object(queue, 'plain', side_effect=Path), \
             patch.object(Path, 'exists', new=lambda path: path.name == 'one.receipt.json'), \
             patch.object(queue, 'read_json', return_value=(receipt, {})), \
             patch.object(queue, 'fingerprint', side_effect=FileNotFoundError('archive missing')), \
             patch.object(queue, 'write_once') as write:
            with self.assertRaises(FileNotFoundError):
                queue.cleanup({}, {}, root, 'one', '2' * 64, [message])
            write.assert_not_called()

    def test_completed_native_deletion_is_reconciled_without_repeat_delete(self):
        root, message, receipt = self.cleanup_case()
        archive_pin = {'path': message['archivePath'], 'bytes': message['bytes'], 'sha256': message['sha256']}
        with patch.object(queue, 'plain', side_effect=Path), \
             patch.object(Path, 'exists', new=lambda path: path.name == 'one.receipt.json'), \
             patch.object(queue, 'read_json', return_value=(receipt, {'sha256': '3' * 64})), \
             patch.object(queue, 'fingerprint', return_value=(archive_pin, b'archive')), \
             patch.dict(queue.os.environ, {'BUILD_URL': 'original-reconciliation-build'}), \
             patch.object(queue, 'write_once') as write, \
             patch.object(queue.subprocess, 'run') as execute:
            result = queue.cleanup({}, {}, root, 'one', '2' * 64, [message])
            self.assertTrue(result['success'])
            self.assertTrue(result['reconciledAfterReceiptFailure'])
            self.assertEqual(result['deletedBytes'], 7)
            write.assert_called_once()
            execute.assert_not_called()


class RecordsTransportCase:
    """Memory evidence and real loopback HTTP; no native owner or filesystem writes."""
    def __init__(self, q, inventory, *, followup=None, original=None, lose_ack=True):
        from http.server import BaseHTTPRequestHandler, HTTPServer
        self.q, self.subject = q, followup or q
        self.repo = Path('E:/Git/ZirconEngine')
        self.raw, self.posts, self.lose_ack = {}, [], lose_ack
        self.api = Mock(CAPABILITY={'enabled': True, 'actions': ['validate'],
                                   'formalAcceptance': False, 'm8Accepted': False},
                        CLIENT_SOURCES=frozenset({'tools/dev/local_cargo.py', 'tools/dev/local-cargo.ps1',
                            'tools/dev/dev-fast-build.ps1', 'tools/dev/submit-jenkins-management.ps1',
                            'tools/jenkins/frontend.py', 'tools/jenkins-management/entry.py'}))
        self.api.capability.side_effect = PermissionError('OpenProcess host ERROR_ACCESS_DENIED')
        self.api.record.return_value = {'payload': original} if original else None
        self.api.document.side_effect = self.document
        manifest = {'schemaVersion': 1, 'files': {'tools/jenkins/member.py':
                    {'bytes': 6, 'sha256': q.sha(b'driver')}}}
        driver_digest = q.sha(q.encoded(manifest))
        driver = self.repo / '.jenkins/runtime/drivers' / driver_digest
        self.driver_root = driver
        self.driver_member = driver / 'tools/jenkins/member.py'
        self.raw[str(self.driver_member)] = b'driver'
        self.put(driver / 'driver-manifest.json', manifest)
        template = "def codex = '@@CODEX@@'\ndef capsule = '@@CAPSULE@@'\ndef runtime = '@@RUNTIME@@'\n"
        members = {'runner.py': b'runner', 'source_preparation.py': b'prepare',
                   'zircon-management.groovy': template.encode()}
        config = {'schemaVersion': 1, 'repositoryRoot': str(self.repo), 'driverDigest': driver_digest,
                  'runtimeOperationId': 'start-current', 'agentLabel': 'zircon-windows-agent',
                  'ordinaryEntry': self.api.CAPABILITY, 'managedWorktree': None,
                  'files': {k: q.sha(v) for k, v in members.items()},
                  'clientSources': {k: '3' * 64 for k in self.api.CLIENT_SOURCES}}
        checksum = q.sha(q.encoded(config))
        capsule = self.repo / '.jenkins/runtime/management' / checksum
        for name, raw in members.items():
            self.raw[str(capsule / name)] = raw
        self.put(capsule / 'capsule.json', config)
        self.put(self.repo / '.jenkins/state/management/installed.json',
                 {'schemaVersion': 1, 'job': 'zircon-management', 'capsule': str(capsule),
                  'capsuleSha256': checksum, 'formalAcceptance': False,
                  **{k: config[k] for k in ('driverDigest', 'runtimeOperationId', 'ordinaryEntry', 'managedWorktree')}})
        self.put(self.repo / '.jenkins/state/deployment/driver.json',
                 {'root': str(driver), 'driverDigest': driver_digest, 'runtimeOperationId': 'start-current'})
        self.put(self.repo / '.jenkins/state/deployment-operation.json',
                 {'operationId': 'start-current', 'generation': 'start-current', 'state': 'running'})
        self.put(self.repo / '.jenkins/state/credentials.json',
                 {'authenticationMode': 'api-token', 'username': 'fixture', 'apiToken': 'fixture'})
        self.jobs = {}
        script = template.replace('@@CODEX@@', 'E:/fixture/codex.exe').replace(
            '@@CAPSULE@@', str(capsule).replace('\\', '/')).replace('@@RUNTIME@@', 'start-current')
        self.jobs['/job/zircon-management/config.xml'] = self.xml(script,
            ('SESSION_ID', 'REQUEST_ID', 'OPERATION_ID', 'ACTION', 'INPUT_JSON',
             'EXPECTED_DRIVER_DIGEST', 'RUNTIME_OPERATION_ID', 'QUERY_ONLY'))
        for subject in ([q, followup] if followup else [q]):
            name = 'records_queue.py' if subject is q else 'records_followup.py'
            template_name = 'zircon-records.groovy' if subject is q else 'zircon-records-followup.groovy'
            text = "def repo = '@@REPO@@'\ndef script = '@@SCRIPT@@'\ndef sha = '@@SOURCE_SHA@@'\n"
            raw = {n: ('fixture-' + n).encode() for n in subject.SOURCES}
            raw[template_name] = text.encode()
            for n, data in raw.items():
                self.raw[str(subject.HERE / n)] = data
            installed = {'job': subject.JOB, 'repositoryRoot': str(self.repo),
                         'driverDigest': driver_digest, 'runtimeOperationId': 'start-current',
                         'sources': {n: q.sha(data) for n, data in raw.items()}}
            location = self.repo / '.jenkins/state/records-queue'
            if subject is not q:
                location /= 'followups'
            self.put(location / 'installed.json', installed)
            for key, value in {'REPO': str(self.repo), 'SCRIPT': str(subject.HERE / name),
                               'SOURCE_SHA': installed['sources'][name]}.items():
                text = text.replace('@@' + key + '@@', value.replace('\\', '/'))
            names = ('INVENTORY_DIGEST', 'TASK_ID', 'MODE') if subject is q else ('FOLLOWUP_DIGEST', 'TASK_ID', 'MODE')
            self.jobs['/job/' + subject.JOB + '/config.xml'] = self.xml(text, names)
        case = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_GET(self):
                raw = (b'{"crumbRequestField":"crumb","crumb":"fixture"}'
                       if self.path == '/crumbIssuer/api/json' else case.jobs[self.path])
                self.send_response(200)
                self.send_header('Content-Length', str(len(raw)))
                self.end_headers()
                self.wfile.write(raw)

            def do_POST(self):
                from urllib.parse import parse_qs
                raw = self.rfile.read(int(self.headers['Content-Length']))
                case.posts.append((self.path, parse_qs(raw.decode())))
                if case.lose_ack:
                    self.close_connection = True
                    return
                self.send_response(201)
                self.send_header('Location', 'http://127.0.0.1:' + str(case.server.server_port) + '/queue/item/17/')
                self.send_header('Content-Length', '0')
                self.end_headers()

        self.server = HTTPServer(('127.0.0.1', 0), Handler)
        self.put(self.repo / '.jenkins/deployment-spec.json',
                 {'controller': {'listenAddress': '127.0.0.1', 'httpPort': self.server.server_port}})
        self.inventory = self.repo / '.jenkins/state/records-queue/fixture-inventory.json'
        self.put(self.inventory, inventory)

    def xml(self, script, parameters):
        return ("<flow-definition><properties><hudson.model.ParametersDefinitionProperty><parameterDefinitions>" +
                ''.join('<p><name>' + n + '</name></p>' for n in parameters) +
                "</parameterDefinitions></hudson.model.ParametersDefinitionProperty></properties>"
                "<definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition'><script>" +
                self.q.escape(script) + "</script><sandbox>true</sandbox></definition>"
                "<disabled>false</disabled></flow-definition>").encode()

    def put(self, path, value):
        self.raw[str(path)] = self.q.encoded(value)

    def document(self, path):
        raw = self.raw[str(path)]
        return json.loads(raw), raw

    def fingerprint(self, path):
        raw = self.raw[str(path)]
        return {'path': str(path), 'sha256': self.q.sha(raw), 'bytes': len(raw), 'mtimeNs': 0}, raw

    def save(self, path, raw):
        if str(path) in self.raw and self.raw[str(path)] != raw:
            raise AssertionError('Immutable evidence changed')
        self.raw[str(path)] = raw

    def __enter__(self):
        from threading import Thread
        import tools.jenkins.state.atomic
        import tools.jenkins.state.locks
        self.thread = Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.stack = ExitStack()
        self.stack.enter_context(patch.object(self.q.sys, 'path', list(self.q.sys.path)))
        self.stack.enter_context(patch.object(self.q, 'entry', return_value=self.api))
        self.stack.enter_context(patch.object(self.q, 'plain', side_effect=Path))
        self.stack.enter_context(patch.object(self.q, 'fingerprint', side_effect=self.fingerprint))
        self.stack.enter_context(patch.object(self.q, 'write_once', side_effect=self.save))
        self.stack.enter_context(patch.object(Path, 'exists', new=lambda p: str(p) in self.raw))
        self.stack.enter_context(patch.object(Path, 'read_bytes', new=lambda p: self.raw[str(p)]))
        self.stack.enter_context(patch.object(Path, 'read_text',
            new=lambda p, **kw: self.raw[str(p)].decode(kw.get('encoding') or 'utf-8')))
        self.stack.enter_context(patch('tools.jenkins.state.atomic.atomic_json',
            side_effect=lambda p, value: self.put(p, value)))
        self.stack.enter_context(patch('tools.jenkins.state.locks.process_lock', return_value=nullcontext()))
        return self

    def __exit__(self, *args):
        try:
            self.stack.close()
        finally:
            self.server.shutdown()
            self.thread.join(5)
            self.server.server_close()
            if self.thread.is_alive():
                raise AssertionError('Fixture HTTP reader did not terminate')


class InstallCase:
    """In-memory installer transport and immutable evidence, with no real writes."""
    def __init__(self, subject, q, *, same_source=False, same_runtime=False):
        self.subject, self.q = subject, q
        self.repo = Path('E:/Git/ZirconEngine')
        self.script_name = 'records_queue.py' if subject.JOB == q.JOB else 'records_followup.py'
        self.template_name = 'zircon-records.groovy' if subject.JOB == q.JOB else 'zircon-records-followup.groovy'
        self.template = ("def repo = '@@REPO@@'\ndef python = '@@PYTHON@@'\n"
                         "def script = '@@SCRIPT@@'\ndef sourceSha = '@@SOURCE_SHA@@'\n"
                         "node('zircon-windows-agent') {}\n").encode()
        self.raw = {name: ('current-' + name).encode() for name in subject.SOURCES}
        self.raw[self.template_name] = self.template.replace(b'\n', b'\r\n')
        self.context = {'repo': self.repo, 'config': {'driverDigest': '2' * 64,
                        'runtimeOperationId': 'start-current'}, 'manager': Mock()}
        self.manager = self.context['manager']
        self.manager.base_url = 'http://127.0.0.1:18080'
        self.api = Mock()
        self.api.capability.return_value = self.context
        self.old = {'schemaVersion': 1, 'job': subject.JOB, 'repositoryRoot': str(self.repo),
                    'driverDigest': self.context['config']['driverDigest'] if same_runtime else '1' * 64,
                    'runtimeOperationId': self.context['config']['runtimeOperationId'] if same_runtime else 'start-old',
                    'sources': {name: q.sha(raw) for name, raw in self.raw.items()},
                    'intakeOnlyAcceptance': True, 'formalAcceptance': False}
        self.old['sources'][self.template_name] = q.sha(self.template)
        if not same_source:
            self.old['sources'][self.script_name] = q.sha(b'previous-owned-script')
        source = self.template.decode()
        for name, value in {'REPO': str(self.repo),
                'PYTHON': str(self.repo / '.jenkins/runtime/python/python.exe'),
                'SCRIPT': str(subject.HERE / self.script_name),
                'SOURCE_SHA': self.old['sources'][self.script_name]}.items():
            source = source.replace('@@' + name + '@@', value.replace('\\', '/'))
        self.old_source = self.current_source = source
        self.installed_path = self.repo / '.jenkins/state/records-queue'
        if subject.JOB != q.JOB:
            self.installed_path /= 'followups'
        self.installed_path /= 'installed.json'
        self.events, self.history, self.published = [], None, None
        self.builds = [{'building': False, 'result': 'ABORTED'}]
        self.items = []
        self.sandbox = 'true'
        self.post_reconciles = True
        self.reads = {}
        self.refresh_previous()

    def refresh_previous(self):
        self.pin = self.q.sha(self.q.encoded(self.old))

    def xml(self):
        return ("<flow-definition><definition "
                "class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition'><script>" +
                self.q.escape(self.current_source) + "</script><sandbox>" + self.sandbox +
                "</sandbox></definition></flow-definition>").encode()

    def request(self, endpoint, *, method='GET', data=None, **kwargs):
        if method == 'POST':
            if endpoint != 'job/' + self.subject.JOB + '/config.xml':
                raise AssertionError('Unexpected mutation: ' + endpoint)
            self.events.append('post')
            import xml.etree.ElementTree as ET
            self.current_source = ET.fromstring(data).findtext('definition/script')
            if not self.post_reconciles:
                self.current_source += '\nforeign change'
            return 200, b''
        if endpoint == 'job/' + self.subject.JOB + '/config.xml':
            return 200, self.xml()
        if endpoint == 'job/' + self.subject.JOB + '/api/json?tree=builds[building,result]':
            return 200, self.q.encoded({'builds': self.builds})
        if endpoint == 'queue/api/json?tree=items[task[name,url]]':
            return 200, self.q.encoded({'items': self.items})
        if endpoint == 'crumbIssuer/api/json':
            return 200, b'{"crumbRequestField":"crumb","crumb":"test"}'
        raise AssertionError('Unexpected read: ' + endpoint)

    def fingerprint(self, path):
        name = Path(path).relative_to(self.subject.HERE).as_posix()
        self.reads[name] = self.reads.get(name, 0) + 1
        raw = self.raw[name]
        return {'sha256': self.q.sha(raw)}, raw

    def read_json(self, path):
        if Path(path) != self.installed_path:
            raise AssertionError('Unexpected evidence read: ' + str(path))
        return copy.deepcopy(self.old), {'sha256': self.pin}

    def write_once(self, path, raw):
        if Path(path) != self.installed_path.parent / 'install-history' / (self.pin + '.json'):
            raise AssertionError('Unexpected immutable write: ' + str(path))
        self.events.append('history')
        self.history = json.loads(raw)

    def publish(self, path, value):
        if Path(path) != self.installed_path:
            raise AssertionError('Unexpected metadata write: ' + str(path))
        self.events.append('publish')
        self.published = copy.deepcopy(value)

    def run(self, *, rebind_runtime=False, **overrides):
        options = {'upgrade_from': self.pin, 'expected_job_script_sha256':
                   self.q.sha(self.current_source.encode()), 'rebind_runtime': rebind_runtime}
        options.update(overrides)
        self.manager._request.side_effect = self.request
        with ExitStack() as stack:
            stack.enter_context(patch.object(self.q, 'fingerprint', side_effect=self.fingerprint))
            stack.enter_context(patch.object(self.q, 'read_json', side_effect=self.read_json))
            stack.enter_context(patch.object(self.q, 'write_once', side_effect=self.write_once))
            stack.enter_context(patch.object(Path, 'exists', new=lambda path: path == self.installed_path))
            stack.enter_context(patch('tools.jenkins.state.atomic.atomic_json', side_effect=self.publish))
            if self.subject is self.q:
                stack.enter_context(patch.object(self.q, 'entry', return_value=self.api))
            else:
                current = {'repositoryRoot': str(self.repo), **self.context['config']}
                stack.enter_context(patch.object(self.q, 'binding', return_value=(self.api, self.context, current)))
            return self.subject.install(self.repo, **options)


class RuntimeRebindTests(unittest.TestCase):
    def test_default_rejects_cross_runtime_and_metadata_only_replacement(self):
        for same_source in (False, True):
            with self.subTest(same_source=same_source):
                case = InstallCase(queue, queue, same_source=same_source)
                with self.assertRaisesRegex(ValueError, '--rebind-runtime'):
                    case.run()
                self.assertEqual(case.events, [])

    def test_explicit_exact_pins_rebind_and_preserve_original_history(self):
        case = InstallCase(queue, queue)
        result = case.run(rebind_runtime=True)
        self.assertEqual(case.events, ['history', 'post', 'publish'])
        self.assertEqual(case.history['previous'], case.old)
        self.assertEqual(case.history['previousInstallationSha256'], case.pin)
        self.assertEqual(case.history['previousJobScriptSha256'], queue.sha(case.old_source.encode()))
        self.assertTrue(case.history['runtimeRebound'])
        self.assertEqual(case.history['next'], result)
        self.assertEqual(case.published, result)
        self.assertEqual(result['driverDigest'], case.context['config']['driverDigest'])
        self.assertEqual(result['runtimeOperationId'], case.context['config']['runtimeOperationId'])
        self.assertEqual(result['sources'][case.template_name], queue.sha(case.raw[case.template_name]))
        self.assertNotEqual(result['sources'][case.template_name], case.old['sources'][case.template_name])
        self.assertFalse(result['formalAcceptance'])

    def test_same_runtime_upgrade_retains_default_exact_pin_route(self):
        case = InstallCase(queue, queue, same_runtime=True)
        case.run()
        self.assertEqual(case.events, ['history', 'post', 'publish'])
        self.assertFalse(case.history['runtimeRebound'])

    def test_stale_prior_or_job_script_pins_reject_before_history(self):
        for override in ({'upgrade_from': '3' * 64}, {'expected_job_script_sha256': '4' * 64}):
            with self.subTest(override=override):
                case = InstallCase(queue, queue)
                with self.assertRaises(ValueError):
                    case.run(rebind_runtime=True, **override)
                self.assertEqual(case.events, [])

    def test_foreign_owner_inventory_and_lexical_template_changes_reject(self):
        for mismatch in ('job', 'repositoryRoot', 'sourceInventory', 'template', 'jobScript', 'sandbox'):
            with self.subTest(mismatch=mismatch):
                case = InstallCase(queue, queue)
                if mismatch == 'job':
                    case.old['job'] = 'foreign-job'
                elif mismatch == 'repositoryRoot':
                    case.old['repositoryRoot'] = 'E:/Git/Foreign'
                elif mismatch == 'sourceInventory':
                    case.old['sources']['foreign.py'] = '5' * 64
                elif mismatch == 'template':
                    case.raw[case.template_name] += b'foreign-code\n'
                elif mismatch == 'jobScript':
                    case.current_source += '\nforeign-code'
                else:
                    case.sandbox = 'false'
                case.refresh_previous()
                with self.assertRaises(ValueError):
                    case.run(rebind_runtime=True)
                self.assertEqual(case.events, [])

    def test_nonterminal_build_and_queued_work_reject_before_history(self):
        for observation in ('building', 'unknownResult', 'queuedName', 'queuedUrl', 'unknownTask'):
            with self.subTest(observation=observation):
                case = InstallCase(queue, queue)
                if observation == 'building':
                    case.builds = [{'building': True, 'result': None}]
                elif observation == 'unknownResult':
                    case.builds = [{'building': False, 'result': None}]
                elif observation == 'queuedName':
                    case.items = [{'task': {'name': queue.JOB, 'url': case.manager.base_url + '/other/'}}]
                elif observation == 'queuedUrl':
                    case.items = [{'task': {'name': 'other', 'url': case.manager.base_url + '/job/' + queue.JOB + '/'}}]
                else:
                    case.items = [{'task': None}]
                with self.assertRaises(ValueError):
                    case.run(rebind_runtime=True)
                self.assertEqual(case.events, [])

    def test_source_or_runtime_changes_during_preflight_reject_before_history(self):
        for changed in ('source', 'health', 'runtime'):
            with self.subTest(changed=changed):
                case = InstallCase(queue, queue)
                if changed == 'source':
                    original = case.fingerprint
                    def fingerprint(path):
                        pin, raw = original(path)
                        if Path(path).name == case.script_name and case.reads[case.script_name] > 1:
                            return {'sha256': queue.sha(b'foreign-source')}, b'foreign-source'
                        return pin, raw
                    case.fingerprint = fingerprint
                elif changed == 'health':
                    case.api.capability.side_effect = [case.context, ValueError('runtime no longer healthy')]
                else:
                    other = {**case.context, 'config': {**case.context['config'], 'runtimeOperationId': 'start-other'}}
                    case.api.capability.side_effect = [case.context, other]
                with self.assertRaises(ValueError):
                    case.run(rebind_runtime=True)
                self.assertEqual(case.events, [])

    def test_post_unknown_or_changed_config_cannot_publish_installation(self):
        for unknown in (True, False):
            with self.subTest(unknown=unknown):
                case = InstallCase(queue, queue)
                if unknown:
                    original = case.request
                    def request(endpoint, **kwargs):
                        if kwargs.get('method') == 'POST':
                            case.events.append('post')
                            raise TimeoutError('unknown acknowledgement')
                        return original(endpoint, **kwargs)
                    case.request = request
                    expected = TimeoutError
                else:
                    case.post_reconciles = False
                    expected = ValueError
                with self.assertRaises(expected):
                    case.run(rebind_runtime=True)
                self.assertEqual(case.events, ['history', 'post'])
                self.assertIsNone(case.published)




class RequirementOutcomeTests(unittest.TestCase):
    def inputs(self, status="input-provided"):
        context = {"repo": Path("E:/Git/ZirconEngine"),
                   "config": {"driverDigest": "1" * 64, "runtimeOperationId": "same-runtime"},
                   "installed": {"capsuleSha256": "2" * 64}}
        original = {"taskId": "one", "inventoryDigest": "3" * 64,
                    "originalIdentity": {"sessionId": "original"},
                    "pendingRequirements": ["complete external input", "actual compile"], "done": False,
                    "formalAcceptance": False}
        value = {"schemaVersion": 1, "taskId": original["taskId"],
                 "originalInventoryDigest": original["inventoryDigest"],
                 "originalIdentity": original["originalIdentity"],
                 "requirementsDigest": queue.sha(queue.encoded(original["pendingRequirements"])),
                 "provider": queue.outcome_provider(context), "formalAcceptance": False,
                 "outcomes": [{"requirementText": original["pendingRequirements"][0],
                     "requirementDigest": queue.sha(original["pendingRequirements"][0].encode()),
                     "status": status, "note": "Exact input supplied; compilation still pending",
                     "evidenceRefs": [{"path": str(context["repo"] / ".jenkins/state/management/evidence.json"),
                                       "sha256": "4" * 64}]}]}
        return context, original, value

    def snapshot_inputs(self):
        context, original, value = self.inputs()
        value.update(schemaVersion=2,
                     requirementsSnapshot=copy.deepcopy(original["pendingRequirements"]))
        return context, original, value

    def test_queued_snapshot_survives_append_without_accepting_new_requirements(self):
        context, original, value = self.snapshot_inputs()
        original["pendingRequirements"].append("actual visual acceptance")
        before = copy.deepcopy(original)
        with patch.object(queue, "fingerprint", return_value=({"sha256": "4" * 64}, b"{}")):
            actual = queue.validate_requirement_outcome(context, value, original)
            queue.attach_requirement_outcome(original, actual, followup_digest="6" * 64, response={})
            viewed = queue.task_outcome_view(context, {"payload": original})
        self.assertEqual(original["pendingRequirements"], before["pendingRequirements"])
        self.assertFalse(original["done"])
        self.assertFalse(original["formalAcceptance"])
        self.assertEqual(viewed["payload"]["currentOutcomeView"]["counts"],
                         {"input-provided": 1, "pending": 2})

    def test_snapshot_cannot_cover_changed_removed_or_reordered_original_requirements(self):
        for requirements in (["changed input", "actual compile"],
                             ["actual compile"],
                             ["actual compile", "complete external input"]):
            with self.subTest(requirements=requirements):
                context, original, value = self.snapshot_inputs()
                original["pendingRequirements"] = requirements
                with self.assertRaisesRegex(ValueError, "original identity"):
                    queue.validate_requirement_outcome(context, value, original)

    def test_snapshot_digest_and_provider_remain_exact_after_append(self):
        for change in ("snapshot", "digest", "provider"):
            with self.subTest(change=change):
                context, original, value = self.snapshot_inputs()
                original["pendingRequirements"].append("new request")
                if change == "snapshot":
                    value["requirementsSnapshot"][0] = "changed input"
                elif change == "digest":
                    value["requirementsDigest"] = "5" * 64
                else:
                    value["provider"]["runtimeOperationId"] = "another-runtime"
                with self.assertRaisesRegex(ValueError, "original identity"):
                    queue.validate_requirement_outcome(context, value, original)

    def test_snapshot_cannot_smuggle_an_outcome_for_a_later_requirement(self):
        context, original, value = self.snapshot_inputs()
        original["pendingRequirements"].append("actual visual acceptance")
        value["outcomes"][0].update(
            requirementText="actual visual acceptance",
            requirementDigest=queue.sha(b"actual visual acceptance"))
        with self.assertRaisesRegex(ValueError, "original requirement"):
            queue.validate_requirement_outcome(context, value, original)

    def test_legacy_partial_projection_still_requires_the_exact_current_inventory(self):
        context, original, value = self.inputs()
        original["pendingRequirements"].append("new request")
        with self.assertRaisesRegex(ValueError, "original identity"):
            queue.validate_requirement_outcome(context, value, original)

    def test_wrong_original_identity_and_changed_requirements_are_rejected(self):
        for field, changed in (("originalIdentity", {"sessionId": "other"}),
                               ("requirementsDigest", "5" * 64)):
            context, original, value = self.inputs()
            value[field] = changed
            with self.assertRaisesRegex(ValueError, "original identity"):
                queue.validate_requirement_outcome(context, value, original)

    def test_input_outcome_preserves_original_pending_and_task_acceptance(self):
        context, original, value = self.inputs()
        before = copy.deepcopy(original)
        with patch.object(queue, "fingerprint", return_value=({"sha256": "4" * 64}, b"{}")):
            actual = queue.validate_requirement_outcome(context, value, original)
        queue.attach_requirement_outcome(original, actual, followup_digest="6" * 64, response={})
        self.assertEqual(original["pendingRequirements"], before["pendingRequirements"])
        self.assertFalse(original["done"])
        self.assertFalse(original["formalAcceptance"])
        with patch.object(queue, "fingerprint", return_value=({"sha256": "4" * 64}, b"{}")):
            viewed = queue.task_outcome_view(context, {"payload": original})
        self.assertEqual(viewed["payload"]["currentOutcomeView"]["counts"],
                         {"input-provided": 1, "pending": 1})

    def test_route_metadata_without_actual_native_and_archive_proof_is_rejected(self):
        context, original, value = self.inputs("route-ready")
        proof = {"status": "ready", **value["provider"], "formalAcceptance": False,
                 "regression": {"path": "regression.json", "sha256": "7" * 64}}
        with patch.object(queue, "fingerprint", return_value=({"sha256": "4" * 64}, queue.encoded(proof))), \
             patch.object(queue, "read_json", return_value=(
                 {"jenkinsResult": "SUCCESS", "nativeProofComplete": False,
                  "archiveByteVerification": []}, {"sha256": "7" * 64})):
            with self.assertRaisesRegex(ValueError, "actual tested deployment"):
                queue.validate_requirement_outcome(context, value, original)

    def test_changed_evidence_is_displayed_stale_without_mutating_original(self):
        context, original, value = self.inputs()
        queue.attach_requirement_outcome(original, value, followup_digest="6" * 64, response={})
        before = copy.deepcopy(original)
        with patch.object(queue, "fingerprint", return_value=({"sha256": "8" * 64}, b"changed")):
            viewed = queue.task_outcome_view(context, {"payload": original})
        self.assertEqual(viewed["payload"]["currentOutcomeView"]["counts"],
                         {"pending": 1, "stale-evidence": 1})
        self.assertEqual(original, before)

    def test_accepted_status_cannot_be_inserted_through_operator_observation(self):
        context, original, value = self.inputs("accepted")
        with self.assertRaisesRegex(ValueError, "nonacceptance status"):
            queue.validate_requirement_outcome(context, value, original)


if __name__ == '__main__':
    unittest.main()
