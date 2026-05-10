'use strict';
const vscode = require('vscode');
const path = require('path');
const fs = require('fs');

const BINARY_NAME = process.platform === 'win32' ? 'sovereign.exe' : 'sovereign';

function resolveBinary(configuredPath) {
    if (configuredPath && configuredPath !== 'auto') {
        const expanded = expandHome(configuredPath);
        if (fs.existsSync(expanded)) return { cmd: expanded, args: [] };
    }

    // Search workspace folders
    const folders = vscode.workspace.workspaceFolders ?? [];
    for (const folder of folders) {
        const found = findInRustWorkspace(folder.uri.fsPath);
        if (found) return { cmd: found, args: [] };
    }

    // Walk up from extension directory to find sovereign-agent rust workspace
    const extDir = path.resolve(__dirname, '..', '..');
    for (const candidate of ancestorRustWorkspaces(extDir)) {
        const found = findInRustWorkspace(candidate);
        if (found) return { cmd: found, args: [] };
    }

    return { cmd: 'cargo', args: ['run', '--manifest-path',
        path.resolve(__dirname, '..', '..', 'rust', 'Cargo.toml'),
        '-p', 'cli', '--'] };
}

function findInRustWorkspace(base) {
    for (const profile of ['release', 'debug']) {
        const candidate = path.join(base, 'rust', 'target', profile, BINARY_NAME);
        if (isExecutable(candidate)) return candidate;
    }
    return null;
}

function ancestorRustWorkspaces(startDir) {
    const results = [];
    let dir = startDir;
    for (let i = 0; i < 6; i++) {
        const parent = path.dirname(dir);
        if (parent === dir) break;
        results.push(parent);
        dir = parent;
    }
    return results;
}

function expandHome(value) {
    const os = require('os');
    if (value === '~') return os.homedir();
    if (value.startsWith('~' + path.sep)) return path.join(os.homedir(), value.slice(2));
    return value;
}

function isExecutable(candidate) {
    try {
        fs.accessSync(candidate, fs.constants.X_OK);
        return true;
    } catch (_) { return false; }
}

module.exports = { resolveBinary };
