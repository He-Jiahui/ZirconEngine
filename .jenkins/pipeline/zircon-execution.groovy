// Shared execution owns the heavy recipe and its native process receipt.
def invokeControl(String action, Map payload) {
    def result
    node('zircon-windows') {
        if (!env.ZIRCON_SEALED_DRIVER || !env.ZIRCON_DRIVER_LAUNCHER || !env.ZIRCON_DRIVER_DIGEST) error('sealed driver environment is required')
        result = load(env.ZIRCON_SEALED_DRIVER).control('execution', action, payload)
    }
    return result
}
def identity = [sessionId: params.SESSION_ID, requestId: params.REQUEST_ID,
                repositoryId: params.REPOSITORY_ID ?: env.ZIRCON_REPOSITORY_ID,
                attemptId: params.ATTEMPT_ID, generation: params.GENERATION as int,
                sourceInputDigest: params.SOURCE_INPUT_DIGEST,
                coverageDigest: params.COVERAGE_DIGEST,
                stageImplementationDigest: env.ZIRCON_DRIVER_DIGEST]
def executionId = params.EXECUTION_ID
if (!identity.repositoryId) error('REPOSITORY_ID/ZIRCON_REPOSITORY_ID is required')
if (!executionId) error('EXECUTION_ID is required')
if (!params.SESSION_ID || !params.REQUEST_ID || !params.ATTEMPT_ID || !params.GENERATION) error('session/request/attempt/generation identity is required')
try {
    stage('Managed execution') {
        def result = invokeControl('run-execution', [identity: identity, executionId: executionId])
        for (int poll = 0; result.status in ['pending', 'running', 'waiting', 'ready'] && poll < 120; poll++) {
            sleep(time: Math.min(30, 2 + poll.intdiv(4)), unit: 'SECONDS')
            if (result.status in ['pending', 'waiting', 'ready']) {
                invokeControl('reconcile-execution', [identity: identity, executionId: executionId])
            }
            result = invokeControl('observe-execution', [identity: identity, executionId: executionId])
        }
        if (!(result.status in ['passed', 'reused', 'complete', 'accepted'])) error("execution did not complete: ${result}")
    }
} finally {
    stage('Finalize execution') {
        invokeControl('finalize-execution', [identity: identity, executionId: executionId])
    }
}
