// Fixed Scripted Pipeline entry. Each control call owns an agent briefly;
// resource waits happen outside node and release the executor.
def invokeControl(String domain, String action, Map payload) {
    def result
    if (!params.RUNTIME_OPERATION_ID || !params.STAGE_IMPLEMENTATION_DIGEST) error('runtime identity parameters are required')
    if (!env.ZIRCON_AGENT_LABEL || params.RUNTIME_OPERATION_ID != env.ZIRCON_RUNTIME_OPERATION_ID || params.STAGE_IMPLEMENTATION_DIGEST != env.ZIRCON_DRIVER_DIGEST) error('runtime identity fence failed')
    // Reuse the exact operation and payload. A busy lock performed no work;
    // all other failures keep their original error/reconciliation behavior.
    for (int attempt = 0; attempt < 12; attempt++) {
        node(env.ZIRCON_AGENT_LABEL) {
            if (!env.ZIRCON_SEALED_DRIVER || !env.ZIRCON_DRIVER_LAUNCHER || !env.ZIRCON_DRIVER_DIGEST) error('sealed driver environment is required')
            result = load(env.ZIRCON_SEALED_DRIVER).control(domain, action, payload)
        }
        if (!(result.status in ['failed', 'blocked'] && result.reasonCode == 'execution_busy' && result.retryable == true)) return result
        if (attempt == 11) error("${domain}/${action} exceeded execution contention retry budget")
        // Release the executor before each wait; at most eleven five-second waits.
        sleep(time: 5, unit: 'SECONDS')
    }
}
def waitControl(String domain, String action, Map payload, int polls = 120) {
    for (int n = 0; n < polls; n++) {
        def result = invokeControl(domain, action, payload)
        if (result.status in ['passed', 'reused', 'complete', 'accepted', 'ready', 'cancelled']) return result
        if (result.status in ['failed', 'blocked']) error("${domain}/${action} blocked: ${result}")
        sleep(time: Math.min(30, 2 + n.intdiv(4)), unit: 'SECONDS')
    }
    error("${domain}/${action} exceeded polling budget")
}
def repositoryId = params.REPOSITORY_ID ?: env.ZIRCON_REPOSITORY_ID
if (!repositoryId) error('REPOSITORY_ID/ZIRCON_REPOSITORY_ID is required')
if (!params.SESSION_ID || !params.REQUEST_ID || !params.ATTEMPT_ID || !params.GENERATION) error('session/request/attempt/generation identity is required')
def identity = [sessionId: params.SESSION_ID, requestId: params.REQUEST_ID,
                repositoryId: repositoryId,
                attemptId: params.ATTEMPT_ID, generation: params.GENERATION as int,
                sourceInputDigest: params.SOURCE_INPUT_DIGEST,
                coverageDigest: params.COVERAGE_DIGEST,
                stageImplementationDigest: env.ZIRCON_DRIVER_DIGEST]
def sealedInputRef = params.SEALED_INPUT_REF
def patchOperationRef = params.PATCH_OPERATION_REF
def patchRequestRef = params.PATCH_REQUEST_REF
def currentSourceRef = params.CURRENT_SOURCE_REF
def sourcePreparationRef = params.SOURCE_PREPARATION_REF
def registeredFlow = false
def executionId = null
def recipeRef = null
def workflowStages = []
def heavyStages = ['build', 'unit_test', 'integration_test', 'regression_test',
                   'package_check', 'editor_exe', 'runtime_dll', 'package']
def needsExecution = false
def commitRequested = false
try {
    if (currentSourceRef && patchRequestRef) error('current source and unapplied patch inputs are mutually exclusive')
    if (currentSourceRef) {
        stage('Prepare current source') {
            def prepared = invokeControl('flow', 'prepare-current-source', [
                identity: [sessionId: identity.sessionId, requestId: identity.requestId,
                           repositoryId: identity.repositoryId, attemptId: identity.attemptId,
                           generation: identity.generation,
                           stageImplementationDigest: identity.stageImplementationDigest],
                currentSourceRef: currentSourceRef,
                runtimeOperationId: params.RUNTIME_OPERATION_ID,
                buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
            if (prepared.status != 'prepared') error("current source was not prepared: ${prepared}")
            def finalIdentity = prepared.identity instanceof Map ? prepared.identity : prepared
            identity.sourceInputDigest = finalIdentity.sourceInputDigest
            identity.coverageDigest = finalIdentity.coverageDigest
            identity.stageImplementationDigest = finalIdentity.stageImplementationDigest ?: identity.stageImplementationDigest
            sealedInputRef = prepared.sealedInputRef ?: finalIdentity.sealedInputRef
            sourcePreparationRef = prepared.sourcePreparationRef ?: finalIdentity.sourcePreparationRef
            if (!identity.sourceInputDigest || !identity.coverageDigest || !sealedInputRef || !sourcePreparationRef) {
                error('prepared current source returned incomplete sealed identity')
            }
        }
    } else if (patchRequestRef) {
        stage('Apply incremental patch') {
            def prepared = invokeControl('flow', 'prepare-patch', [
                identity: [sessionId: identity.sessionId, requestId: identity.requestId,
                           repositoryId: identity.repositoryId, attemptId: identity.attemptId,
                           generation: identity.generation,
                           stageImplementationDigest: identity.stageImplementationDigest],
                patchRequestRef: patchRequestRef,
                buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
            if (prepared.status != 'prepared') error("incremental patch was not prepared: ${prepared}")
            def finalIdentity = prepared.identity instanceof Map ? prepared.identity : prepared
            identity.sourceInputDigest = finalIdentity.sourceInputDigest
            identity.coverageDigest = finalIdentity.coverageDigest
            identity.stageImplementationDigest = finalIdentity.stageImplementationDigest ?: identity.stageImplementationDigest
            sealedInputRef = prepared.sealedInputRef ?: finalIdentity.sealedInputRef
            patchOperationRef = prepared.patchOperationRef ?: finalIdentity.patchOperationRef
            sourcePreparationRef = prepared.sourcePreparationRef ?: finalIdentity.sourcePreparationRef
            patchRequestRef = prepared.patchRequestRef ?: patchRequestRef
            if (!identity.sourceInputDigest || !identity.coverageDigest || !sealedInputRef ||
                (!patchOperationRef && !sourcePreparationRef) || (patchOperationRef && sourcePreparationRef)) {
                error('prepared patch returned incomplete sealed identity')
            }
        }
    } else if (!identity.sourceInputDigest || !identity.coverageDigest || !sealedInputRef) {
        error('legacy flow requires SOURCE_INPUT_DIGEST, COVERAGE_DIGEST and SEALED_INPUT_REF')
    }
    stage('Register request') {
        def registered = invokeControl('flow', 'register-flow', [repositoryId: repositoryId,
            sessionId: identity.sessionId, requestId: identity.requestId,
            attemptId: identity.attemptId, generation: identity.generation,
            sourceInputDigest: identity.sourceInputDigest, coverageDigest: identity.coverageDigest,
            driverDigest: identity.stageImplementationDigest, identity: identity,
            sealedInputRef: sealedInputRef,
            patchOperationRef: patchOperationRef,
            patchRequestRef: patchRequestRef,
            currentSourceRef: currentSourceRef,
            sourcePreparationRef: sourcePreparationRef,
            buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
        executionId = registered.executionId
        registeredFlow = true
    }
    stage('Classify and compose') {
        def composePayload = [identity: identity, patchOperationRef: patchOperationRef,
                              sourcePreparationRef: sourcePreparationRef]
        def composed = invokeControl('flow', 'compose-flow', composePayload)
        if (!(composed.template instanceof String) || !composed.template ||
            !(composed.stages instanceof List) || composed.stages.isEmpty() ||
            composed.stages.any { !(it instanceof String) || !it } ||
            composed.stages.toSet().size() != composed.stages.size()) {
            error('compose-flow returned an incomplete authoritative DAG')
        }
        def supportedStages = ['source', 'patch', 'syntax', 'format', 'review',
                               'acceptance', 'commit', 'notify', 'gc'] + heavyStages
        if (composed.stages.any { !(it in supportedStages) }) {
            error('compose-flow returned an unsupported DAG stage')
        }
        workflowStages = composed.stages.collect { it }
        if (!workflowStages.contains('acceptance')) {
            error('Flow requires an acceptance stage; maintenance uses its independent job')
        }
        env.WORKFLOW_TEMPLATE = composed.template
        needsExecution = workflowStages.any { it in heavyStages }
        commitRequested = workflowStages.contains('commit') && params.ALLOW_COMMIT?.toBoolean()
    }
    if (workflowStages.contains('source')) {
        stage('Current source receipt') {
            invokeControl('flow', 'run-stage', [identity: identity, stage: 'source',
                sourcePreparationRef: sourcePreparationRef])
        }
    }
    if (workflowStages.contains('patch')) {
        stage('Patch receipt') {
            invokeControl('flow', 'run-stage', [identity: identity, stage: 'patch',
                patchOperationRef: patchOperationRef, sourcePreparationRef: sourcePreparationRef])
        }
    }
    if (workflowStages.contains('syntax')) {
        stage('Syntax') {
            invokeControl('flow', 'run-stage', [identity: identity, stage: 'syntax',
                sealedInputRef: sealedInputRef, edition: params.RUST_EDITION ?: '2021'])
        }
    }
    if (workflowStages.contains('format')) {
        stage('Format') {
            invokeControl('flow', 'run-stage', [identity: identity, stage: 'format',
                sealedInputRef: sealedInputRef, edition: params.RUST_EDITION ?: '2021'])
        }
    }
    if (workflowStages.contains('review')) {
        stage('Review') {
            invokeControl('flow', 'run-stage', [identity: identity, stage: 'review',
                sealedInputRef: sealedInputRef, edition: params.RUST_EDITION ?: '2021'])
        }
    }
    if (needsExecution) {
        stage('Submit execution') {
            def submitted = invokeControl('flow', 'submit-execution', [identity: identity, executionId: executionId,
                patchOperationRef: patchOperationRef, sealedInputRef: sealedInputRef,
                sourcePreparationRef: sourcePreparationRef,
                template: env.WORKFLOW_TEMPLATE, buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
            if (submitted.executionId) executionId = submitted.executionId
            if (submitted.recipeRef) recipeRef = submitted.recipeRef
            if (!executionId) error('submit-execution returned no authoritative executionId')
            if (!recipeRef) error('submit-execution returned no authoritative recipeRef')
            invokeControl('flow', 'ensure-execution-job', [identity: identity,
                executionId: executionId, recipeRef: recipeRef,
                buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
        }
    }
    if (needsExecution) {
        stage('Await execution receipts') {
            def observed = waitControl('flow', 'observe-flow', [identity: identity, executionId: executionId])
            if (observed.status == 'cancelled') {
                currentBuild.result = 'ABORTED'
                error('This execution consumer was cancelled')
            }
        }
    }
    if (workflowStages.contains('acceptance')) {
        stage('Acceptance') {
            invokeControl('flow', 'run-stage', [identity: identity, stage: 'acceptance', executionId: executionId,
                flowId: "${identity.sessionId}:${identity.requestId}:${identity.attemptId}"])
        }
    }
    if (commitRequested) {
        stage('Authorized Git commit') {
            def workflowKey = "${identity.sessionId}:${identity.requestId}:${identity.attemptId}"
            invokeControl('integration', 'commit-flow', [identity: identity,
                workflowKey: workflowKey, executionId: executionId,
                authorization: params.COMMIT_AUTHORIZATION])
        }
    }
    def notifyRequested = workflowStages.contains('notify') && params.ALLOW_NOTIFY?.toBoolean()
    def finalized
    stage('Finalize authorized actions') {
        finalized = invokeControl('flow', 'finalize-flow', [identity: identity,
            executionId: executionId, commitRequested: commitRequested,
            notifyRequested: notifyRequested, notifyDestination: params.NOTIFY_DESTINATION])
    }
    if (notifyRequested && finalized.notifyEligible == true) {
        stage('Notify') {
            def eventId = "${identity.sessionId}:${identity.requestId}:${identity.attemptId}:complete"
            try {
                invokeControl('notification', 'deliver', [identity: identity,
                    repositoryId: identity.repositoryId, sessionId: identity.sessionId,
                    eventId: eventId, destinationId: params.NOTIFY_DESTINATION,
                    destinationProof: params.NOTIFY_DESTINATION])
                invokeControl('flow', 'finalize-flow', [identity: identity,
                    executionId: executionId, commitRequested: commitRequested,
                    notifyRequested: true, notifyDestination: params.NOTIFY_DESTINATION])
            } catch (notifyErr) {
                echo "notification requires independent retry or reconciliation: ${notifyErr}"
            }
        }
    }
} catch (err) {
    def origin = err.toString()
    try {
        if (!registeredFlow && currentSourceRef) {
            invokeControl('current-source', 'query', [identity: identity, currentSourceRef: currentSourceRef,
                runtimeOperationId: params.RUNTIME_OPERATION_ID,
                buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
        } else if (!registeredFlow && patchRequestRef) {
            invokeControl('flow', 'reconcile-patch', [identity: identity, patchRequestRef: patchRequestRef,
                patchOperationRef: patchOperationRef, error: origin,
                buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
        } else {
            invokeControl('flow', 'reconcile-flow', [identity: identity, executionId: executionId, error: origin])
        }
    } catch (reconcileErr) {
        echo "reconcile failed after original error: ${reconcileErr}"
    }
    throw err
} finally {
    // Preserve the original validation result if maintenance needs a retry.
    // Paths and native proofs come from the exact original task in State.
    stage('Inspect task cache') {
        try {
            def observed = invokeControl('maintenance', 'post-task-cache', [identity: identity, phase: 'assess'])
            if (observed.result?.cleanupRequired == true && observed.result?.gcAuthorized == true) {
                def cleaned = invokeControl('maintenance', 'post-task-cache', [identity: identity, phase: 'gc'])
                echo "task cache: ${cleaned.result?.status}; reclaimed bytes: ${cleaned.result?.reclaimedBytes ?: 0}"
            } else {
                echo "task cache: ${observed.result?.scratch?.status ?: 'protected'}"
            }
        } catch (cacheErr) {
            echo "task cache inspection requires independent retry or reconciliation: ${cacheErr}"
        }
    }
}
