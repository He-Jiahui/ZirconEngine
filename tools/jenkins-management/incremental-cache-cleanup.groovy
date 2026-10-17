// BEGIN ZIRCON INCREMENTAL CACHE CLEANUP
node('zircon-windows-agent') {
    timeout(time: 15, unit: 'MINUTES') {
        def cacheRunId = 'incremental-' + env.BUILD_NUMBER
        def cacheReport = 'E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/incremental/' + cacheRunId
        def dry = params.DRY_RUN == true ? ' --dry-run' : ''
        dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-scheduled-cleanup') {
            def status = bat(returnStatus: true, label: 'Discard unlocked Rust incremental sessions under disk pressure',
                script: '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
                '"E:/Git/ZirconEngine/tools/jenkins-management/incremental_cache_cleanup.py" run --run-id "' +
                cacheRunId + '"' + dry + '\r\nexit /b %ERRORLEVEL%')
            for (String name : ['receipt.json', 'artifacts.json']) {
                if (fileExists(cacheReport + '/' + name)) {
                    writeFile(file: 'incremental-' + name, text: readFile(file: cacheReport + '/' + name, encoding: 'UTF-8'), encoding: 'UTF-8')
                    archiveArtifacts(artifacts: 'incremental-' + name, fingerprint: false)
                }
            }
            if (status != 0) error('Rust cache cleanup failed; inspect the original receipt')
            def cacheSummary = readJson(readFile(file: cacheReport + '/receipt.json', encoding: 'UTF-8'))
            env.ZIRCON_INCREMENTAL_DELETED_BYTES = cacheSummary.logicalDeletedBytes.toString()
            env.ZIRCON_INCREMENTAL_RECEIPT = cacheReport + '/receipt.json'
            if (cacheSummary.status == 'partial') currentBuild.result = 'UNSTABLE'
            echo('Rust incremental cache: deleted ' + cacheSummary.logicalDeletedBytes +
                ' logical bytes; unlinked allocation ' + cacheSummary.unlinkedAllocationBytes +
                ' bytes; receipt ' + env.ZIRCON_INCREMENTAL_RECEIPT)
        }
    }
}
// END ZIRCON INCREMENTAL CACHE CLEANUP
