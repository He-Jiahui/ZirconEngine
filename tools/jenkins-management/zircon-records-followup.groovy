def repo = '@@REPO@@'
def python = '@@PYTHON@@'
def script = '@@SCRIPT@@'
def sourceSha = '@@SOURCE_SHA@@'
if (!(params.MODE in ['attach', 'self-test'])) error('Unsupported supplementary intake mode')
if (!(params.TASK_ID ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/) || !(params.FOLLOWUP_DIGEST ==~ /[0-9a-f]{64}/)) error('Invalid supplementary identity')
node('zircon-windows-agent') {
    dir(repo + '/.jenkins/agent/workspace/zircon-records-followup') {
        if (params.MODE == 'self-test') {
            bat('"' + python + '" -B -X utf8 "' + script + '" self-test --repo-root "' + repo + '" --expected-source-sha256 ' + sourceSha)
        } else {
            currentBuild.description = params.TASK_ID + ': supplementary intake; original validation remains pending'
            bat('"' + python + '" -B -X utf8 "' + script + '" work --repo-root "' + repo + '" --task-id ' + params.TASK_ID +
                ' --followup-digest ' + params.FOLLOWUP_DIGEST + ' --expected-source-sha256 ' + sourceSha)
            def output = repo + '/.jenkins/state/records-queue/followups/controls/' + params.FOLLOWUP_DIGEST + '.json'
            writeFile(file: params.FOLLOWUP_DIGEST + '.receipt.json', text: readFile(file: output, encoding: 'UTF-8'), encoding: 'UTF-8')
            archiveArtifacts(artifacts: params.FOLLOWUP_DIGEST + '.receipt.json', fingerprint: true)
        }
    }
}
