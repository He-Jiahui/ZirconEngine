def repo = '@@REPO@@'
def python = '@@PYTHON@@'
def script = '@@SCRIPT@@'
def sourceSha = '@@SOURCE_SHA@@'
if (!(params.MODE in ['task', 'reconcile', 'self-test'])) error('Unsupported fixed records mode')
if (params.MODE == 'self-test') {
    node('zircon-windows-agent') {
        dir(repo + '/.jenkins/agent/workspace/zircon-records') {
            bat('"' + python + '" -B -X utf8 "' + script + '" self-test --repo-root "' + repo + '" --expected-source-sha256 ' + sourceSha)
        }
    }
    return
}
if (!(params.TASK_ID ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/) || !(params.INVENTORY_DIGEST ==~ /[0-9a-f]{64}/)) error('Invalid task identity')
currentBuild.description = params.TASK_ID + ': intake, pending formal acceptance'
def done = false
while (!done) {
    node('zircon-windows-agent') {
        dir(repo + '/.jenkins/agent/workspace/zircon-records') {
            def output = repo + '/.jenkins/state/records-queue/controls/' + params.TASK_ID + '.' + env.BUILD_NUMBER + '.json'
            def rc = bat(returnStatus: true, script: '"' + python + '" -B -X utf8 "' + script + '" work --repo-root "' + repo +
                '" --task-id ' + params.TASK_ID + ' --inventory-digest ' + params.INVENTORY_DIGEST + ' --output-file "' + output +
                '" --expected-source-sha256 ' + sourceSha)
            if (!(rc in [0, 10])) error('Records intake requires reconciliation; original messages and receipts are retained')
            done = rc == 0
            writeFile(file: params.TASK_ID + '.receipt.json', text: readFile(file: output, encoding: 'UTF-8'), encoding: 'UTF-8')
            archiveArtifacts(artifacts: params.TASK_ID + '.receipt.json', fingerprint: true)
            currentBuild.description = params.TASK_ID + ': ' + (done ? 'evidence archived' : 'waiting for formal input/acceptance')
        }
    }
    // Waiting contracts persist in Jenkins without occupying an executor.
    if (!done) sleep(time: 30, unit: 'MINUTES')
}
