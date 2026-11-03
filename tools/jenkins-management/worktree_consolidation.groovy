// ZIRCON BOUNDED WORKTREE CONSOLIDATION
if (params.ACTION == 'consolidate-worktrees') {
    if (!(params.OPERATION_ID ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error('Invalid consolidation operation')
    timeout(time: 20, unit: 'MINUTES') {
        node('zircon-windows-agent') {
            dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-management') {
                def input = "E:/Git/ZirconEngine/.jenkins/state/controls/consolidation-${params.OPERATION_ID}.${env.BUILD_NUMBER}.json"
                writeFile(file: input, text: params.INPUT_JSON, encoding: 'UTF-8')
                int code
                withEnv(['INPUT_JSON=']) {
                    code = powershell(returnStatus: true, script: '''
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath '@@SOURCE@@' -Algorithm SHA256).Hash.ToLowerInvariant() -ne '@@SHA@@') { throw 'Consolidation source changed' }
& 'E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe' -B -X utf8 '@@SOURCE@@' controller --input-file ''' + "'${input}'" + '''
exit $LASTEXITCODE
''')
                }
                currentBuild.description = "consolidation ${params.OPERATION_ID}"
                def payload = parseJson(params.INPUT_JSON)
                if (!(payload.taskId ==~ /[A-Za-z0-9][A-Za-z0-9_.-]{0,79}/)) error('Invalid report task')
                dir('E:/cargo-targets/zircon-local/reports/' + payload.taskId + '/' + params.OPERATION_ID) {
                    archiveArtifacts(artifacts: '*.json,*.jsonl,*.bin', allowEmptyArchive: true, fingerprint: true)
                }
                if (code != 0) error('Consolidation requires original outcome reconciliation')
            }
        }
    }
    return
}
