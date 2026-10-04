// Trusted, fixed glue. Source files are data and never become Pipeline code.
node('zircon-windows') {
    timeout(time: 20, unit: 'MINUTES') {
        stage('Receive sealed source') {
            deleteDir()
            unstash 'SOURCE_BUNDLE'
            env.PILOT_BUNDLE_PATH = "${pwd()}/SOURCE_BUNDLE"
        }
        try {
            stage('Run declared validation') {
                bat label: 'Zircon isolated pilot', script: '''@echo off
"%JENKINS_PILOT_PYTHON%" -B -m tools.jenkins.pilot.runner --root "%JENKINS_PILOT_ROOT%" --repo-root "%ZIRCON_REPO_ROOT%" --bundle "%PILOT_BUNDLE_PATH%" --build-number "%BUILD_NUMBER%" --job "%JOB_NAME%" --monitor-jenkins
exit /b %ERRORLEVEL%
'''
            }
        } finally {
            archiveArtifacts artifacts: 'receipt.json,output.log,managed-request.json,managed-result.json,managed-evidence.json,registry-seed.json', allowEmptyArchive: false, fingerprint: true
        }
    }
}
