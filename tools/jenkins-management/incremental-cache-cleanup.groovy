// BEGIN ZIRCON INCREMENTAL CACHE CLEANUP
node('zircon-windows-agent') {
    timeout(time: 15, unit: 'MINUTES') {
        def cacheRunId = 'incremental-' + env.BUILD_NUMBER
        long totalCacheDeleted = 0L
        dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-scheduled-cleanup') {
          def nativeBinding = powershell(returnStdout: true, label: 'Verify real current cleanup owner before cache deletion', script: '''
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath '@RUNTIME@' -Algorithm SHA256).Hash.ToLowerInvariant() -ne '@RUNTIME_SHA@') { throw 'Cleanup native binding changed' }
if ((Get-FileHash -LiteralPath '@PYTHON@' -Algorithm SHA256).Hash.ToLowerInvariant() -ne '@PYTHON_SHA@') { throw 'Cleanup interpreter changed' }
& '@PYTHON@' -B -X utf8 '@RUNTIME@' --repo-root 'E:/Git/ZirconEngine' --expected-driver-digest '@DRIVER@'
if ($LASTEXITCODE -ne 0) { throw 'Current Jenkins owner identity is unverified; no cache deletion' }
''')
          writeFile(file: 'incremental-runtime-binding.json', text: nativeBinding, encoding: 'UTF-8')
          archiveArtifacts(artifacts: 'incremental-runtime-binding.json', fingerprint: false)
          for (String cacheKey : @CACHE_KEYS@) {
            def suffix = cacheKey == 'shared-e' ? '' : '-' + cacheKey
            def cacheReport = 'E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/incremental/' + cacheRunId + suffix
            def status
            withEnv(['ZIRCON_CACHE_RUN_ID=' + cacheRunId, 'ZIRCON_CACHE_KEY=' + cacheKey,
                     'ZIRCON_CACHE_DRY_RUN=' + (params.DRY_RUN == true ? '1' : '0')]) {
              status = powershell(returnStatus: true, label: 'Discard unlocked Rust incremental sessions: ' + cacheKey,
                script: '''
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath '@ENTRY@' -Algorithm SHA256).Hash.ToLowerInvariant() -ne '@ENTRY_SHA@') { throw 'Cleanup implementation changed; preserve caches' }
if ((Get-FileHash -LiteralPath '@HELPER@' -Algorithm SHA256).Hash.ToLowerInvariant() -ne '@HELPER_SHA@') { throw 'Cleanup helper changed; preserve caches' }
$cacheArguments = @('run', '--run-id', $env:ZIRCON_CACHE_RUN_ID, '--cache-key', $env:ZIRCON_CACHE_KEY)
if ($env:ZIRCON_CACHE_DRY_RUN -eq '1') { $cacheArguments += '--dry-run' }
& '@PYTHON@' -B -X utf8 '@ENTRY@' @cacheArguments
exit $LASTEXITCODE
''')
            }
            for (String name : ['receipt.json', 'artifacts.json']) {
                if (fileExists(cacheReport + '/' + name)) {
                    def archived = 'incremental-' + cacheKey + '-' + name
                    writeFile(file: archived, text: readFile(file: cacheReport + '/' + name, encoding: 'UTF-8'), encoding: 'UTF-8')
                    archiveArtifacts(artifacts: archived, fingerprint: false)
                }
            }
            if (status != 0) error('Rust cache cleanup failed; inspect the original receipt')
            def cacheSummary = readJson(readFile(file: cacheReport + '/receipt.json', encoding: 'UTF-8'))
            totalCacheDeleted += cacheSummary.logicalDeletedBytes as long
            env.ZIRCON_INCREMENTAL_DELETED_BYTES = totalCacheDeleted.toString()
            env.ZIRCON_INCREMENTAL_RECEIPT = cacheReport + '/receipt.json'
            if (cacheSummary.status == 'partial') currentBuild.result = 'UNSTABLE'
            echo('Rust incremental ' + cacheKey + ': ' + cacheSummary.status + '; deleted ' + cacheSummary.logicalDeletedBytes +
                ' logical bytes; unlinked allocation ' + cacheSummary.unlinkedAllocationBytes +
                '; retained sessions ' + cacheSummary.retainedSessions + '; receipt ' + env.ZIRCON_INCREMENTAL_RECEIPT)
          }
        }
    }
}
// END ZIRCON INCREMENTAL CACHE CLEANUP
