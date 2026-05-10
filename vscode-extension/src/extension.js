'use strict';
const vscode = require('vscode');
const { ChatController } = require('./chatController');

let controller = null;

function getOrCreateController(context) {
    if (!controller) {
        controller = new ChatController(context);
    }
    return controller;
}

function activate(context) {
    context.subscriptions.push(
        vscode.commands.registerCommand('sovereignAgent.openChat', () => {
            getOrCreateController(context).show();
        }),
        vscode.commands.registerCommand('sovereignAgent.restart', () => {
            getOrCreateController(context)._startSession();
        }),
        vscode.commands.registerCommand('sovereignAgent.stop', () => {
            controller?._session?.stop();
        }),
        vscode.commands.registerCommand('sovereignAgent.clear', () => {
            controller?._postWebview({ type: 'clear' });
            if (controller) controller._transcript = [];
        }),
    );
}

function deactivate() {
    controller?.dispose();
    controller = null;
}

module.exports = { activate, deactivate };
