// Gap-fix Pipeline: claim validation → patch → cargo check → accept gap issue.
// Reuses the invokeControl/waitControl pattern from zircon-flow.groovy.
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
if (!params.ISSUE_ID)     error('ISSUE_ID is required')
if (!params.SESSION_ID)   error('SESSION_ID is required')
if (!params.REQUEST_ID)   error('REQUEST_ID is required')
if (!params.ATTEMPT_ID)   error('ATTEMPT_ID is required')
if (!params.GENERATION)   error('GENERATION is required')

def identity = [
    sessionId:              params.SESSION_ID,
    requestId:              params.REQUEST_ID,
    repositoryId:           repositoryId,
    attemptId:              params.ATTEMPT_ID,
    generation:             params.GENERATION as int,
    sourceInputDigest:      params.SOURCE_INPUT_DIGEST,
    coverageDigest:         params.COVERAGE_DIGEST,
    stageImplementationDigest: env.ZIRCON_DRIVER_DIGEST,
]
def issueId       = params.ISSUE_ID
def patchRequestRef = params.PATCH_REQUEST_REF
def buildRoot     = params.BUILD_ROOT ?: env.ZIRCON_BUILD_ROOT
def receiptRef    = null

try {
    stage('Validate claim') {
        // Confirm the issue is claimed by this session before any heavy work.
        def validated = invokeControl('gap', 'validate-claim', [
            issueId:   issueId,
            sessionId: params.SESSION_ID,
        ])
        if (validated.status != 'claimed') error("Gap issue ${issueId} is not claimed by session ${params.SESSION_ID}: ${validated}")
    }

    if (patchRequestRef) {
        stage('Apply incremental patch') {
            def prepared = invokeControl('flow', 'prepare-patch', [
                identity:        [sessionId: identity.sessionId, requestId: identity.requestId,
                                  repositoryId: identity.repositoryId, attemptId: identity.attemptId,
                                  generation: identity.generation,
                                  stageImplementationDigest: identity.stageImplementationDigest],
                patchRequestRef: patchRequestRef,
                buildRoot:       buildRoot,
            ])
            if (prepared.status != 'prepared') error("incremental patch was not prepared: ${prepared}")
            identity.sourceInputDigest = prepared.identity?.sourceInputDigest ?: prepared.sourceInputDigest
            identity.coverageDigest    = prepared.identity?.coverageDigest    ?: prepared.coverageDigest
            patchRequestRef = prepared.patchRequestRef ?: patchRequestRef
        }
    }

    stage('Register gap flow') {
        invokeControl('flow', 'register-flow', [
            repositoryId:     repositoryId,
            sessionId:        identity.sessionId,
            requestId:        identity.requestId,
            attemptId:        identity.attemptId,
            generation:       identity.generation,
            sourceInputDigest: identity.sourceInputDigest,
            coverageDigest:   identity.coverageDigest,
            driverDigest:     identity.stageImplementationDigest,
            identity:         identity,
            sealedInputRef:   params.SEALED_INPUT_REF,
            patchRequestRef:  patchRequestRef,
            buildRoot:        buildRoot,
        ])
    }

    stage('Classify') {
        invokeControl('flow', 'compose-flow', [identity: identity, patchOperationRef: patchRequestRef])
    }

    stage('Syntax') {
        invokeControl('flow', 'run-stage', [identity: identity, stage: 'syntax',
            sealedInputRef: params.SEALED_INPUT_REF, edition: params.RUST_EDITION ?: '2021'])
    }

    stage('Format') {
        invokeControl('flow', 'run-stage', [identity: identity, stage: 'format',
            sealedInputRef: params.SEALED_INPUT_REF, edition: params.RUST_EDITION ?: '2021'])
    }

    stage('Submit Cargo execution') {
        def submitted = invokeControl('flow', 'submit-execution', [
            identity:        identity,
            patchRequestRef: patchRequestRef,
            sealedInputRef:  params.SEALED_INPUT_REF,
            buildRoot:       buildRoot,
        ])
        if (!submitted.executionId) error('submit-execution returned no authoritative executionId')
        identity.executionId = submitted.executionId
        invokeControl('flow', 'ensure-execution-job', [
            identity:    identity,
            executionId: submitted.executionId,
            recipeRef:   submitted.recipeRef,
            buildRoot:   buildRoot,
        ])
    }

    stage('Await execution') {
        def observed = waitControl('flow', 'observe-flow', [identity: identity, executionId: identity.executionId])
        if (observed.status == 'cancelled') {
            currentBuild.result = 'ABORTED'
            error('Gap execution consumer was cancelled')
        }
    }

    stage('Acceptance') {
        def accepted = invokeControl('flow', 'run-stage', [identity: identity, stage: 'acceptance',
            executionId: identity.executionId,
            flowId: "${identity.sessionId}:${identity.requestId}:${identity.attemptId}"])
        receiptRef = accepted.receipt?.toString() ?: "accepted:${identity.requestId}:${identity.attemptId}"
    }

    stage('Accept gap issue') {
        invokeControl('gap', 'accept', [
            issueId:    issueId,
            sessionId:  params.SESSION_ID,
            receiptRef: receiptRef,
        ])
        echo "Gap issue ${issueId} accepted with receipt ${receiptRef}"
    }

    stage('Finalize') {
        invokeControl('flow', 'finalize-flow', [
            identity:         identity,
            executionId:      identity.executionId,
            commitRequested:  params.ALLOW_COMMIT?.toBoolean(),
            notifyDestination: params.NOTIFY_DESTINATION ?: 'default',
        ])
    }

} catch (err) {
    def origin = err.toString()
    try {
        invokeControl('flow', 'reconcile-flow', [
            identity:    identity,
            executionId: identity.executionId,
            error:       origin,
        ])
    } catch (reconcileErr) {
        echo "gap reconcile failed after original error: ${reconcileErr}"
    }
    throw err
}
