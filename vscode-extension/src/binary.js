'use strict';
const vscode = require('vscode');
const path = require('path');
const fs = require('fs');

function resolveBinary(configuredPath) {
    if (configuredPath && configuredPath !== 'auto' && fs.existsSync(configuredPath)) {
        return { cmd: configuredPath, args: [] };
    }
    const folders = vscode.workspace.workspaceFolders ?? [];
    for (const folder of folders) {
        for (const profile of ['release', 'debug']) {
            const candidate = path.join(folder.uri.fsPath, 'rust', 'target', profile, 'sovereign');
            if (fs.existsSync(candidate)) return { cmd: candidate, args: [] };
        }
    }
    return { cmd: 'cargo', args: ['run', '-p', 'sovereign', '--'] };
}

module.exports = { resolveBinary };
