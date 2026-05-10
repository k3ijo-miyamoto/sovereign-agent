'use strict';
const path = require('path');
const vscode = require('vscode');
const { workspaceRoot } = require('./workspace');

async function saveTranscript(messages) {
    const root = workspaceRoot();
    const targetDir = path.join(root, '.sovereign', 'transcripts');
    const stamp = new Date().toISOString().replace(/[:.]/g, '-');
    const targetPath = path.join(targetDir, 'transcript-' + stamp + '.md');
    const content = renderTranscript(messages);
    await vscode.workspace.fs.createDirectory(vscode.Uri.file(targetDir));
    await vscode.workspace.fs.writeFile(vscode.Uri.file(targetPath), Buffer.from(content, 'utf8'));
    return path.relative(root, targetPath);
}

function renderTranscript(messages) {
    const lines = ['# Sovereign Agent Transcript', ''];
    for (const msg of messages) {
        const text = String(msg.text || '').trimEnd();
        if (!text) continue;
        lines.push('## ' + (msg.role === 'user' ? 'User' : 'Assistant'), '', text, '');
    }
    return lines.join('\n') + '\n';
}

module.exports = { saveTranscript, renderTranscript };
