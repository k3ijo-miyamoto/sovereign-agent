'use strict';
const vscode = require('vscode');
const { ChatController } = require('./chatController');

let controller = null;

function activate(context) {
    context.subscriptions.push(
        vscode.commands.registerCommand('sovereignAgent.openChat', () => {
            if (!controller) {
                controller = new ChatController(context);
            }
            controller.show();
        })
    );
}

function deactivate() {
    controller?.dispose();
    controller = null;
}

module.exports = { activate, deactivate };
