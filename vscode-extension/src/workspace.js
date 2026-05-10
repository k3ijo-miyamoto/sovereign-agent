'use strict';
const path = require('path');
const fs = require('fs');
const os = require('os');

function workspaceRoot() {
    try {
        const vscode = require('vscode');
        const folder = vscode.workspace.workspaceFolders?.[0];
        if (folder) return folder.uri.fsPath;
    } catch (_) {}
    return path.resolve(__dirname, '..', '..');
}

function expandHome(value) {
    if (value === '~') return os.homedir();
    if (value?.startsWith('~' + path.sep)) return path.join(os.homedir(), value.slice(2));
    return value;
}

function isExecutable(candidate) {
    try {
        fs.accessSync(candidate, fs.constants.X_OK);
        return true;
    } catch (_) { return false; }
}

module.exports = { workspaceRoot, expandHome, isExecutable };
