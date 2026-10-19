// Independent Jenkins timer. The selected sealed maintenance owner decides GC.
@NonCPS
def plain(Object value) {
    if (value instanceof Map) {
        def result = new LinkedHashMap()
        for (def entry : value.entrySet()) result.put(entry.key.toString(), plain(entry.value))
        return result
    }
    if (value instanceof List) return value.collect { plain(it) }
    return value
}
@NonCPS
def readJson(String text) { return plain(new groovy.json.JsonSlurper().parseText(text)) }

def maintenanceAgent(Closure body) {
    timeout(time: 5, unit: 'MINUTES') {
        node('zircon-windows-agent') { body() }
    }
}

def runtimeFence() {
    if (env.NODE_NAME != 'zircon-windows-agent') error('scheduled cleanup requires the fixed node')
    powershell(label: 'Verify selected maintenance runtime', script: '''
$ErrorActionPreference = 'Stop'
$selected = Get-Content -LiteralPath "$env:ZIRCON_REPO_ROOT/.jenkins/state/deployment/driver.json" -Raw | ConvertFrom-Json
if ($selected.driverDigest -ne $env:ZIRCON_DRIVER_DIGEST -or $selected.runtimeOperationId -ne $env:ZIRCON_RUNTIME_OPERATION_ID) {
    throw 'Scheduled cleanup driver/runtime changed; reinstall the job for the selected runtime'
}
if ($selected.steps -ne $env:ZIRCON_SEALED_DRIVER -or $selected.launcher -ne $env:ZIRCON_DRIVER_LAUNCHER) {
    throw 'Scheduled cleanup sealed paths changed'
}
''')
}
def bindScheduledRuntime() {
    def binding
    maintenanceAgent {
        if (env.NODE_NAME != 'zircon-windows-agent') error('scheduled cleanup requires the fixed node')
        binding = readJson(powershell(returnStdout: true, label: 'Bind new cleanup run to verified native runtime', script: '''
$ErrorActionPreference = 'Stop'
$entry = Join-Path $env:ZIRCON_REPO_ROOT 'tools/dev/jenkins_cleanup_runtime.py'
if ((Get-FileHash -LiteralPath $entry -Algorithm SHA256).Hash.ToLowerInvariant() -ne $env:ZIRCON_CLEANUP_BINDING_SHA256) { throw 'Cleanup runtime binding implementation changed; reinstall reviewed job' }
if ((Get-FileHash -LiteralPath $env:JENKINS_PYTHON -Algorithm SHA256).Hash.ToLowerInvariant() -ne $env:ZIRCON_CLEANUP_PYTHON_SHA256) { throw 'Cleanup job interpreter changed' }
& $env:JENKINS_PYTHON -B -X utf8 $entry --repo-root $env:ZIRCON_REPO_ROOT --expected-driver-digest $env:ZIRCON_DRIVER_DIGEST
if ($LASTEXITCODE -ne 0) { throw 'Cannot bind cleanup to the verified native runtime' }
'''))
    }
    if (binding.driverDigest != env.ZIRCON_DRIVER_DIGEST || !binding.runtimeOperationId ||
            !(binding.nativeOwners instanceof Map) || !(binding.endpoint instanceof Map)) {
        error('new cleanup runtime proof is incomplete')
    }
    binding.put('installedRuntimeOperationId', env.ZIRCON_RUNTIME_OPERATION_ID)
    env.ZIRCON_RUNTIME_OPERATION_ID = binding.runtimeOperationId
    return binding
}

def control(String action, Map payload) {
    def result
    for (int attempt = 0; attempt < 12; attempt++) {
        maintenanceAgent {
            runtimeFence()
            def nonce = 'control-' + env.BUILD_NUMBER + '-' + UUID.randomUUID().toString()
            def input = env.ZIRCON_REPO_ROOT + '/.jenkins/state/controls/' + nonce + '.json'
            def bound = new LinkedHashMap(payload)
            bound.put('runtimeOperationId', env.ZIRCON_RUNTIME_OPERATION_ID)
            bound.put('driverDigest', env.ZIRCON_DRIVER_DIGEST)
            writeFile(file: input, text: groovy.json.JsonOutput.toJson(bound))
            withEnv(['ZIRCON_SCHEDULED_INPUT=' + input, 'ZIRCON_SCHEDULED_ACTION=' + action]) {
                try {
                    result = readJson(powershell(returnStdout: true, label: 'Bounded maintenance response: ' + action, script: '''
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath $env:ZIRCON_SCHEDULED_CONTROL_ENTRY -Algorithm SHA256).Hash.ToLowerInvariant() -ne $env:ZIRCON_SCHEDULED_CONTROL_SHA256) { throw 'Scheduled control implementation changed' }
& $env:JENKINS_PYTHON -B -X utf8 $env:ZIRCON_SCHEDULED_CONTROL_ENTRY control --plan $env:ZIRCON_SCHEDULED_CONTROL_PLAN --plan-sha256 $env:ZIRCON_SCHEDULED_CONTROL_PLAN_SHA256 --self-sha256 $env:ZIRCON_SCHEDULED_CONTROL_SHA256 --input-file $env:ZIRCON_SCHEDULED_INPUT --action $env:ZIRCON_SCHEDULED_ACTION
if ($LASTEXITCODE -ne 0) { throw 'Original maintenance invocation failed; inspect retained response/native proof' }
'''))
                } finally {
                    ws('E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/control/' + env.BUILD_NUMBER) {
                        archiveArtifacts(artifacts: nonce + '/**/*.json,' + nonce + '/**/*.bin', allowEmptyArchive: true, fingerprint: false)
                    }
                }
            }
        }
        if (!(result.status in ['failed', 'blocked'] && result.reasonCode == 'execution_busy' && result.retryable == true)) {
            if (result.status != 'accepted' || !(result.result instanceof Map)) error('maintenance did not return an accepted result')
            return result.result
        }
        if (attempt == 11) error('scheduled cleanup exceeded the contention retry budget')
        sleep(time: 5, unit: 'SECONDS')
    }
}

def diskSpaceSnapshot() {
    def result
    maintenanceAgent {
        runtimeFence()
        result = readJson(powershell(returnStdout: true, label: 'Measure maintenance disk headroom', script: '''
$ErrorActionPreference = 'Stop'
@(foreach ($drive in @('D', 'E', 'F')) {
    $volume = [IO.DriveInfo]::new($drive + ':\')
    if (-not $volume.IsReady) { throw 'Maintenance volume is unavailable' }
    [long]$free = $volume.AvailableFreeSpace
    if ($free -lt 0 -or $free -gt $volume.TotalSize) { throw 'Invalid native disk space observation' }
    @{drive = $drive; freeBytes = $free; totalBytes = [long]$volume.TotalSize}
}) | ConvertTo-Json -Compress
'''))
    }
    return result
}
def selectCandidates(Map inventory, String root, String selectionId, int maxObjects, long maxBytes) {
    def candidates
    maintenanceAgent {
        runtimeFence()
        def selectionFile = env.ZIRCON_REPO_ROOT + '/.jenkins/state/controls/' + selectionId + '.json'
        writeFile(file: selectionFile, text: groovy.json.JsonOutput.toJson([
            buildRoot: root, inventoryRef: inventory.responseRef, inventoryOperationId: inventory.operationId,
            maxObjects: maxObjects, maxBytes: maxBytes]))
        withEnv(['ZIRCON_CLEANUP_SELECTION=' + selectionFile]) {
            candidates = readJson(powershell(returnStdout: true, label: 'Select bounded registered CAS objects', script: '''
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath $env:ZIRCON_SCHEDULED_CONTROL_ENTRY -Algorithm SHA256).Hash.ToLowerInvariant() -ne $env:ZIRCON_SCHEDULED_CONTROL_SHA256) { throw 'Scheduled control implementation changed' }
& $env:JENKINS_PYTHON -B -X utf8 $env:ZIRCON_SCHEDULED_CONTROL_ENTRY select --plan $env:ZIRCON_SCHEDULED_CONTROL_PLAN --plan-sha256 $env:ZIRCON_SCHEDULED_CONTROL_PLAN_SHA256 --self-sha256 $env:ZIRCON_SCHEDULED_CONTROL_SHA256 --input-file $env:ZIRCON_CLEANUP_SELECTION
if ($LASTEXITCODE -ne 0) { throw 'Original inventory reference or bounded selection failed' }
'''))
        }
    }
    return candidates
}

def requestId = 'scheduled-cleanup-' + UUID.randomUUID().toString()
def identity = [repositoryId: env.ZIRCON_REPOSITORY_ID,
                sessionId: env.ZIRCON_MAINTENANCE_SESSION, requestId: requestId]
def report = [schemaVersion: 1, requestId: requestId, identity: identity,
              driverDigest: env.ZIRCON_DRIVER_DIGEST, runtimeOperationId: env.ZIRCON_RUNTIME_OPERATION_ID,
              dryRun: params.DRY_RUN == true, maxObjects: 64, maxBytes: 268435456,
              plannedBytes: 0L, reclaimedBytes: 0L, roots: [], status: 'running',
              scope: 'registered-CAS-objects-only', minimumFreeBytes: 37580963840L,
              startedAt: new Date().format("yyyy-MM-dd'T'HH:mm:ssXXX", TimeZone.getTimeZone('UTC')),
              preserved: ['warm preparation/target/compiler caches', 'active and historical references', 'unregistered objects']]
int remainingObjects = 64
long remainingBytes = 268435456L
try {
    report.put('runtimeBinding', bindScheduledRuntime())
    report.runtimeOperationId = env.ZIRCON_RUNTIME_OPERATION_ID
    report.put('diskFreeBefore', diskSpaceSnapshot())
    for (String drive : ['D', 'E', 'F']) {
        def root = drive + ':\\cargo-targets'
        def operation = requestId + '-' + drive.toLowerCase()
        def entry = [buildRoot: root, inventoryOperationId: operation + '-inventory',
                     gcOperationId: operation + '-gc', plannedBytes: 0L, reclaimedBytes: 0L, status: 'inventory']
        report.roots.add(entry)
        def payload = [identity: identity, buildRoot: root, ownedPaths: [],
                       runtimeOperationId: env.ZIRCON_RUNTIME_OPERATION_ID]
        def inventory
        stage('Inventory ' + drive) {
            inventory = control('inventory', new LinkedHashMap(payload) + [operationId: entry.inventoryOperationId])
        }
        if (inventory.operationId != entry.inventoryOperationId || inventory.buildRoot != root ||
                !(inventory.protectedDigestCount instanceof Number) || !(inventory.unknownDigestCount instanceof Number) ||
                !(inventory.responseRef instanceof Map)) {
            error('maintenance inventory differs from the original root/operation')
        }
        entry.put('unknownProtectedCount', inventory.unknownDigestCount)
        entry.put('protectedDigestCount', inventory.protectedDigestCount)
        entry.put('inventoryResponseRef', inventory.responseRef)
        def selected = selectCandidates(inventory, root, operation + '-selection', remainingObjects, remainingBytes)
        entry.put('candidateCount', selected.candidateCount)
        entry.put('plannedBytes', selected.plannedBytes)
        report.plannedBytes += selected.plannedBytes
        remainingObjects -= (selected.candidateCount as int)
        remainingBytes -= (selected.plannedBytes as long)
        if (report.dryRun) {
            entry.status = 'dry-run'
        } else {
            stage('Bounded GC ' + drive) {
                entry.status = 'gc-requested'
                def collected = control('gc', new LinkedHashMap(payload) + [
                    operationId: entry.gcOperationId,
                    cleanupScope: [digests: selected.digests, maxObjects: 64, maxBytes: 268435456]])
                if (collected.operationId != entry.gcOperationId || collected.buildRoot != root ||
                        !(collected.removed instanceof List) || !(collected.reclaimedBytes instanceof Number) ||
                        collected.reclaimedBytes < 0 || collected.reclaimedBytes > selected.plannedBytes ||
                        collected.unknownProtected != true) error('GC receipt differs from the bounded maintenance request')
                entry.put('removed', collected.removed)
                entry.put('gcResponseRef', collected.responseRef)
                entry.put('reclaimedBytes', collected.reclaimedBytes)
                report.reclaimedBytes += collected.reclaimedBytes
                entry.status = 'collected'
            }
        }
    }
    report.status = report.dryRun ? 'dry-run' : 'complete'
} catch (err) {
    report.status = 'failed'
    report.reason = err.toString()
    report.put('unknownGcOperations', report.roots.findAll { it.status == 'gc-requested' }.collect { it.gcOperationId })
    currentBuild.description = 'failed; original maintenance ' + requestId
    throw err
} finally {
    try {
        report.put('diskFreeAfter', diskSpaceSnapshot())
        report.put('diskPressure', report.diskFreeAfter.findAll { it.freeBytes < report.minimumFreeBytes })
    } catch (observationError) {
        report.put('diskSnapshotError', observationError.toString())
        if (report.status != 'failed') {
            report.status = 'failed'
            report.reason = observationError.toString()
            currentBuild.result = 'FAILURE'
        }
    }
    report.put('finishedAt', new Date().format("yyyy-MM-dd'T'HH:mm:ssXXX", TimeZone.getTimeZone('UTC')))
    if (report.status in ['complete', 'dry-run'] && report.diskPressure && !report.diskPressure.isEmpty()) {
        report.status = 'needs-review'
        currentBuild.result = 'UNSTABLE'
    }
    maintenanceAgent {
        ws('E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup') {
            def name = 'scheduled-cleanup-' + env.BUILD_NUMBER + '.json'
            def ledger = 'scheduled-cleanup-' + env.BUILD_NUMBER + '.artifacts.json'
            writeFile(file: ledger, text: groovy.json.JsonOutput.toJson([schemaVersion: 1,
                taskId: 'jenkins-scheduled-cleanup', owner: requestId,
                artifacts: [[path: pwd() + '/' + name, retainUntil: 'original maintenance and references reconciled']]]))
            writeFile(file: name, text: groovy.json.JsonOutput.prettyPrint(groovy.json.JsonOutput.toJson(report)))
            archiveArtifacts(artifacts: name + ',' + ledger, fingerprint: true)
        }
        currentBuild.description = 'compiler cache ' + (env.ZIRCON_INCREMENTAL_DELETED_BYTES ?: '0') + ' logical bytes; ' + report.status + '; reclaimed ' + report.reclaimedBytes + ' logical bytes; ' + requestId
        echo('Scheduled cleanup: planned=' + report.plannedBytes + ', reclaimed=' + report.reclaimedBytes + ', status=' + report.status)
    }
}
