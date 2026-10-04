// Fixed Scripted Pipeline entry. Each control call owns an agent briefly;
// resource waits happen outside node and release the executor.
def invokeControl(String domain, String action, Map payload) {
    def result
    node('zircon-windows') {
        if (!env.ZIRCON_SEALED_DRIVER || !env.ZIRCON_DRIVER_LAUNCHER || !env.ZIRCON_DRIVER_DIGEST) error('sealed driver environment is required')
        result = load(env.ZIRCON_SEALED_DRIVER).control(domain, action, payload)
    }
    return result
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
def registeredFlow = false
def executionId = null
def recipeRef = null
try {
    if (patchRequestRef) {
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
            patchRequestRef = prepared.patchRequestRef ?: patchRequestRef
            if (!identity.sourceInputDigest || !identity.coverageDigest || !sealedInputRef || !patchOperationRef) {
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
            buildRoot: params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT])
        executionId = registered.executionId
        registeredFlow = true
    }
    stage('Classify and compose') {
        def composePayload = [identity: identity, patchOperationRef: patchOperationRef]
        def composed = invokeControl('flow', 'compose-flow', composePayload)
        env.WORKFLOW_TEMPLATE = composed.template ?: params.TEMPLATE ?: 'auto'
    }
    stage('Patch receipt') {
        invokeControl('flow', 'run-stage', [identity: identity, stage: 'patch',
            patchOperationRef: patchOperationRef])
    }
    stage('Syntax') {
        invokeControl('flow', 'run-stage', [identity: identity, stage: 'syntax',
            sealedInputRef: sealedInputRef, edition: params.RUST_EDITION ?: '2021'])
    }
    stage('Format') {
        invokeControl('flow', 'run-stage', [identity: identity, stage: 'format',
            sealedInputRef: sealedInputRef, edition: params.RUST_EDITION ?: '2021'])
    }
    if (env.WORKFLOW_TEMPLATE != 'comments_only') {
        stage('Submit execution') {
            def submitted = invokeControl('flow', 'submit-execution', [identity: identity, executionId: executionId,
                patchOperationRef: patchOperationRef, sealedInputRef: sealedInputRef,
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
    if (env.WORKFLOW_TEMPLATE != 'comments_only') {
        stage('Await execution receipts') {
            def observed = waitControl('flow', 'observe-flow', [identity: identity, executionId: executionId])
            if (observed.status == 'cancelled') {
                currentBuild.result = 'ABORTED'
                error('This execution consumer was cancelled')
            }
        }
    }
    stage('Acceptance') {
        invokeControl('flow', 'run-stage', [identity: identity, stage: 'acceptance', executionId: executionId,
            flowId: "${identity.sessionId}:${identity.requestId}:${identity.attemptId}"])
    }
    if (params.ALLOW_COMMIT?.toBoolean()) {
        stage('Authorized Git commit') {
            def workflowKey = "${identity.sessionId}:${identity.requestId}:${identity.attemptId}"
            invokeControl('integration', 'commit-flow', [identity: identity,
                workflowKey: workflowKey, executionId: executionId,
                authorization: params.COMMIT_AUTHORIZATION])
        }
    }
    stage('Notify') {
        // Deliver any enqueued notification for this flow. The outbox is
        // durable: a delivery failure here does not roll back the acceptance
        // or commit; it leaves the record in 'unknown' state for reconciliation.
        def eventId = "${identity.sessionId}:${identity.requestId}:${identity.attemptId}:complete"
        try {
            invokeControl('notification', 'deliver', [identity: identity,
                eventId: eventId,
                destinationId: params.NOTIFY_DESTINATION ?: 'default',
                destinationProof: params.NOTIFY_DESTINATION ?: 'default'])
        } catch (notifyErr) {
            echo "notify deliver non-fatal: ${notifyErr}"
        }
    }
    stage('Finalize authorized actions') {
        invokeControl('flow', 'finalize-flow', [identity: identity,
            executionId: executionId, commitRequested: params.ALLOW_COMMIT?.toBoolean()])
    }
} catch (err) {
    def origin = err.toString()
    try {
        if (!registeredFlow && patchRequestRef) {
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
}
