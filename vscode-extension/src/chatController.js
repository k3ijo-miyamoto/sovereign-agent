'use strict';
const vscode = require('vscode');
const path = require('path');
const { Session } = require('./session');
const { getWebviewHtml } = require('./webviewHtml');
const settings = require('./settings');
const { createStatusController } = require('./status');
const { saveWebviewImage, looksLikeClipboardImageRequest } = require('./images');
const { saveTranscript } = require('./transcript');
const { workspaceRoot } = require('./workspace');

const TASK_PROMPTS = {
    docstring: (file, lang) =>
        'Add a clear docstring to every function in `' + file + '` that lacks one.\n' +
        'Requirements: do NOT change function signatures or bodies. Language: ' + lang + '.',
    tests: (file, lang) =>
        'Generate comprehensive ' + (lang === 'python' ? 'pytest' : 'unit') + ' tests for the functions in `' + file + '`.\n' +
        'Save the test file alongside the source file.',
    typeHints: (file) =>
        'Add type annotations to all function parameters and return values in `' + file + '`.\n' +
        'Do NOT change function bodies.',
    commitMsg: () =>
        'Run `git diff --staged` and generate a concise conventional-commit message for the staged changes.',
};

class ChatController {
    constructor(context) {
        this._context = context;
        this._panel = null;
        this._session = null;
        this._transcript = [];
        this._status = createStatusController();
        this._lastEditorUri = null;

        context.subscriptions.push(
            vscode.window.onDidChangeActiveTextEditor(editor => {
                if (editor) this._lastEditorUri = editor.document.uri;
            }),
            this._status.item,
        );
    }

    async show() {
        if (this._panel) { this._panel.reveal(); return; }

        const overrides = settings.getSensitiveWorkspaceOverrides();
        if (overrides.length > 0) {
            const choice = await vscode.window.showWarningMessage(
                '.vscode/settings.json が機密設定を上書きしています: ' + overrides.join(', ') + '\n' +
                '悪意ある設定によりコードが外部サーバーに送信される可能性があります。続行しますか？',
                { modal: true },
                '続行する',
            );
            if (choice !== '続行する') return;
        }

        this._panel = vscode.window.createWebviewPanel(
            'sovereignAgent', 'Sovereign Agent',
            vscode.ViewColumn.Beside,
            { enableScripts: true, retainContextWhenHidden: true }
        );

        this._panel.webview.html = getWebviewHtml();
        this._startSession();

        this._panel.webview.onDidReceiveMessage(msg => this._onWebviewMessage(msg));
        this._panel.onDidDispose(() => {
            this._session?.dispose();
            this._session = null;
            this._panel = null;
            this._status.update('idle');
        });
    }

    _startSession(overrideModel) {
        this._session?.dispose();
        this._status.update('starting');
        const cfg = settings.get();
        this._session = new Session(cfg, overrideModel || null, evt => this._handleEvent(evt));
        this._session.start();
    }

    async _onWebviewMessage(msg) {
        switch (msg.type) {
            case 'send': {
                const text = (msg.text || '').trim();
                if (!text) return;
                this._transcript.push({ role: 'user', text });
                this._postWebview({ type: 'user', text });
                this._postWebview({ type: 'thinking' });
                this._status.update('busy');
                this._session.send(text);
                break;
            }
            case 'image': {
                await this._handleImage(msg);
                break;
            }
            case 'task': {
                const prompt = await this._buildTaskPrompt(msg.task);
                if (!prompt) return;
                const model = settings.getTaskModel(msg.task);
                this._transcript.push({ role: 'user', text: prompt });
                this._postWebview({ type: 'user', text: '[' + msg.task + '] ' + prompt.slice(0, 80) + '…' });
                this._postWebview({ type: 'thinking' });
                this._status.update('busy');
                if (model !== settings.get().model) {
                    this._startSession(model);
                }
                this._session.send(prompt);
                break;
            }
            case 'restart': {
                this._transcript = [];
                this._startSession();
                this._postWebview({ type: 'clear' });
                this._postWebview({ type: 'system', text: '— CLI を再起動しました —' });
                break;
            }
            case 'stop': {
                this._session?.stop();
                this._status.update('stopped');
                this._postWebview({ type: 'ready' });
                break;
            }
            case 'clear': {
                this._transcript = [];
                this._postWebview({ type: 'clear' });
                break;
            }
            case 'export': {
                await this._exportTranscript();
                break;
            }
            case 'get_settings': {
                const cfg = settings.get();
                const models = cfg.provider === 'ollama'
                    ? await settings.fetchOllamaModels(cfg.baseUrl)
                    : [];
                this._postWebview({ type: 'settings', settings: cfg, models });
                break;
            }
            case 'save_settings': {
                await settings.save(msg.settings);
                this._startSession();
                this._postWebview({ type: 'system', text: '— 設定を保存し CLI を再起動しました —' });
                break;
            }
        }
    }

    async _buildTaskPrompt(task) {
        const uri = vscode.window.activeTextEditor?.document.uri ?? this._lastEditorUri;
        if (!uri && task !== 'commitMsg') {
            vscode.window.showWarningMessage('ファイルを開いてからタスクボタンを押してください');
            return null;
        }
        const filePath = uri ? path.basename(uri.fsPath) : '';
        const lang = uri ? (vscode.window.activeTextEditor?.document.languageId ?? '') : '';
        const fn = TASK_PROMPTS[task];
        return fn ? fn(filePath, lang) : null;
    }

    async _handleImage(msg) {
        if (!msg.images?.length) return;
        try {
            const imagePath = await saveWebviewImage(msg.images[0]);
            const text = (msg.text || '').trim();
            const prompt = text
                ? 'Image saved at ' + imagePath + '. ' + text
                : 'An image was saved at ' + imagePath + '. Please describe its contents.';
            this._transcript.push({ role: 'user', text: prompt });
            this._postWebview({ type: 'user', text: '[image] ' + (text || '画像を送信しました') });
            this._postWebview({ type: 'thinking' });
            this._status.update('busy');
            this._session.send(prompt);
        } catch (err) {
            this._postWebview({ type: 'error', message: '画像の保存に失敗しました: ' + (err.message || err) });
        }
    }

    _handleEvent(evt) {
        switch (evt.type) {
            case 'text': {
                this._postWebview({ type: 'text_delta', delta: evt.delta });
                const last = this._transcript[this._transcript.length - 1];
                if (last?.role === 'assistant') { last.text += evt.delta; }
                else { this._transcript.push({ role: 'assistant', text: evt.delta }); }
                break;
            }
            case 'tool_start':
                this._postWebview({ type: 'tool_start', name: evt.name });
                break;
            case 'tool_done':
                this._postWebview({ type: 'tool_done', name: evt.name, ok: evt.ok });
                break;
            case 'ready':
                this._status.update('ready');
                this._postWebview({ type: 'ready' });
                break;
            case 'error':
                this._status.update('error');
                this._postWebview({ type: 'error', message: evt.message });
                break;
        }
    }

    async _exportTranscript() {
        try {
            const savedPath = await saveTranscript(this._transcript);
            this._postWebview({ type: 'system', text: '— トランスクリプトを保存しました: ' + savedPath + ' —' });
            vscode.window.showInformationMessage('トランスクリプトを保存しました: ' + savedPath);
        } catch (err) {
            this._postWebview({ type: 'error', message: 'エクスポートに失敗しました: ' + (err.message || err) });
        }
    }

    _postWebview(msg) {
        this._panel?.webview.postMessage(msg);
    }

    dispose() {
        this._session?.dispose();
        this._panel?.dispose();
        this._status.dispose();
    }
}

module.exports = { ChatController };
