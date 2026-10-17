// ZIRCON DURABLE CHECKOUT COMMIT FOLLOWUP
timeout(time: 10, unit: 'MINUTES') {
  node('zircon-windows-agent') {
    if (env.NODE_NAME != 'zircon-windows-agent') error('Unexpected Git followup node')
    dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-git-followup') {
      def report = "E:/cargo-targets/zircon-local/reports/jenkins-git-delivery-20261017/followup/${env.BUILD_NUMBER}.json"
      def code = bat(returnStatus: true, script:
        '@echo off\r\n"E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe" -B -X utf8 ' +
        '"E:/Git/ZirconEngine/tools/jenkins-management/checkout_commit.py" followup ' +
        '--output-file "' + report + '"\r\nexit /b %ERRORLEVEL%')
      if (!fileExists(report)) error('Git followup produced no receipt')
      def response = readFile(file: report, encoding: 'UTF-8')
      writeFile(file: 'git-followup.json', text: response, encoding: 'UTF-8')
      if (code != 0 && code != 3) {
        archiveArtifacts(artifacts: 'git-followup.json', fingerprint: false)
        error('Git followup failed; inspect its receipt')
      }
      def description = report.replace('.json', '.description.txt')
      if (!fileExists(description)) error('Git followup produced no description')
      def summary = readFile(file: description, encoding: 'UTF-8').trim()
      writeFile(file: 'git-followup-description.txt', text: summary, encoding: 'UTF-8')
      archiveArtifacts(artifacts: 'git-followup.json,git-followup-description.txt', fingerprint: false)
      currentBuild.description = summary
      if (code == 3) currentBuild.result = 'UNSTABLE'
    }
  }
}

