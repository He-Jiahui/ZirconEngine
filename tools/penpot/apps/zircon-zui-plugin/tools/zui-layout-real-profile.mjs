import assert from 'node:assert/strict';
import { randomBytes, randomUUID } from 'node:crypto';
import { createConnection } from 'node:net';
import { writeFile } from 'node:fs/promises';
import { dirname } from 'node:path';
import { requireRealSessionDirectory } from './zui-layout-real-session-contract.mjs';

/** The protocol used by Penpot 2.17's official backend/scripts/manage.py. */
export async function createRealPenpotProfile(preplUrl, credentialsPath) {
  requireRealSessionDirectory(dirname(credentialsPath));
  const endpoint = new URL(preplUrl);
  assert.equal(endpoint.protocol, 'tcp:');
  assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(endpoint.hostname));
  assert.ok(endpoint.port, 'Use the actual local backend management port');
  const credentials = { fullname: 'Zircon real persistence verification',
    email: `zircon-real-${randomUUID()}@example.test`, password: randomBytes(32).toString('base64url') };
  const profile = await new Promise((resolveProfile, reject) => {
    const socket = createConnection({ host: endpoint.hostname.replace(/^\[|\]$/g, ''), port: Number(endpoint.port) });
    let buffered = '';
    let completed = false;
    const fail = (error) => { completed = true; socket.destroy(); reject(error); };
    socket.setTimeout(30_000, () => fail(new Error('Official Penpot management create-profile timed out')));
    socket.on('error', fail);
    socket.on('connect', () => socket.write(`${JSON.stringify({ cmd: 'create-profile', params: credentials })}\n`));
    socket.on('data', (chunk) => {
      buffered += chunk.toString('utf8');
      let newline;
      while ((newline = buffered.indexOf('\n')) >= 0) {
        const line = buffered.slice(0, newline);
        buffered = buffered.slice(newline + 1);
        let result;
        try { result = JSON.parse(line); } catch { fail(new Error('Official management returned invalid JSON')); return; }
        // Management stdout is deliberately not forwarded: it may contain account data.
        if (result.tag === 'out') continue;
        if (result.tag !== 'ret' || result.err) { fail(new Error('Official management create-profile failed')); return; }
        if (!result.val?.id || result.val.email !== credentials.email) { fail(new Error('Official management did not return the created account')); return; }
        completed = true;
        socket.end();
        resolveProfile(result.val);
        return;
      }
    });
    socket.on('close', () => { if (!completed) reject(new Error('Official management closed before account creation acknowledgement')); });
  });
  await writeFile(credentialsPath, `${JSON.stringify(credentials)}\n`, { flag: 'wx', mode: 0o600 });
  return { credentialsPath, profileId: profile.id };
}
