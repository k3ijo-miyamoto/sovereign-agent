'use strict';
const vscode = require('vscode');

function createStatusController() {
    const item = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    item.command = 'sovereignAgent.openChat';
    item.text = 'Sovereign: idle';
    item.tooltip = 'Open Sovereign Agent chat';
    item.show();

    return {
        item,
        update(state) {
            const labels = {
                starting: '$(loading~spin) Sovereign: starting',
                ready:    'Sovereign: ready',
                busy:     '$(loading~spin) Sovereign: thinking...',
                stopped:  'Sovereign: stopped',
                error:    'Sovereign: error',
            };
            item.text = labels[state] || 'Sovereign: idle';
        },
        dispose() { item.dispose(); },
    };
}

module.exports = { createStatusController };
