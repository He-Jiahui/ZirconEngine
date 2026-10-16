"""Run the scoped checkout commit regression under Jenkins, without hash gates."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path('E:/Git/ZirconEngine')
OUTPUT = Path('E:/cargo-targets/zircon-local/reports/jenkins-commit-audit-20261017/tests')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-id', required=True)
    parser.add_argument('--suite', choices=('all', 'support'), default='all')
    args = parser.parse_args()
    if not re.fullmatch(r'checkout-commit-[A-Za-z0-9_.-]{1,64}', args.run_id):
        raise ValueError('Invalid regression run ID')
    if os.environ.get('JOB_NAME') != 'zircon-management-tests' or os.environ.get('NODE_NAME') != 'zircon-windows-agent':
        raise ValueError('Regression requires the Jenkins management test job')
    spec = importlib.util.spec_from_file_location('commit_checks_helper', ROOT / 'tools/jenkins-management/checkout_commit.py')
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    output = helper.physical(OUTPUT / args.run_id)
    if output.exists():
        raise ValueError('Retain the existing regression run; use another test run ID')
    output.mkdir(parents=True)
    fixtures = helper.physical(output / 'fixtures')
    manifest = {'task_id': 'jenkins-commit-audit-20261017', 'owner': 'zircon-management-tests',
                'source_checkout': str(ROOT), 'status': 'running',
                'dependencies': [str(sys.executable), 'Git for Windows'],
                'temporary_artifacts': [str(fixtures)],
                'retained_artifacts': [str(output / name) for name in ('artifacts.json', 'summary.json', 'stdout.txt', 'stderr.txt')],
                'retention': 'Keep test result and process exit evidence with the repair report'}
    helper.write_json(output / 'artifacts.json', manifest)
    fixtures.mkdir()
    environment = dict(os.environ, ZR_COMMIT_TEST_ROOT=str(fixtures), TEMP=str(fixtures), TMP=str(fixtures),
                       TMPDIR=str(fixtures), PYTHONDONTWRITEBYTECODE='1', ZR_COMMIT_NATIVE='1')
    start = time.time()
    command = [sys.executable, '-B', '-X', 'utf8', str(ROOT / 'tools/jenkins-management/tests/test_checkout_commit.py')]
    if args.suite == 'support':
        command.append('CommitEntrySupportTests')
    with (output / 'stdout.txt').open('wb') as stdout, (output / 'stderr.txt').open('wb') as stderr:
        process = subprocess.Popen(command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr)
        try:
            code = process.wait(timeout=1200)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
            raise
    leftovers = [str(path) for path in fixtures.iterdir()]
    if not leftovers:
        fixtures.rmdir()
    summary = {'status': 'passed' if code == 0 and not leftovers else 'failed', 'exitCode': code,
               'pid': process.pid, 'waitedForExit': True, 'startedAt': start, 'finishedAt': time.time(),
               'suite': args.suite, 'fixturesRetained': leftovers, 'formalAcceptance': False, 'sourceDigestRequired': False}
    helper.write_json(output / 'summary.json', summary)
    manifest.update(status='complete' if summary['status'] == 'passed' else 'needs-review',
                    temporary_artifacts=leftovers, active_owned_processes=[], summary=summary)
    helper.write_json(output / 'artifacts.json', manifest)
    print(json.dumps(summary))
    return 0 if summary['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
