'use strict';
const vscode = require('vscode');

// Defaults from CLAUDE.md routing plan (Phase A)
const TASK_DEFAULTS = {
    docstring:  'gemma3:12b',
    tests:      'qwen3:14b',
    typeHints:  'gemma3:12b',
    commitMsg:  'qwen3:8b-nothink',
};

// Keys that must never be overridden by .vscode/settings.json (security)
const SENSITIVE_KEYS = ['provider', 'baseUrl'];

function getTaskModel(taskId) {
    const c = vscode.workspace.getConfiguration('sovereignAgent');
    const configured = c.get('taskModel.' + taskId, '');
    return configured || TASK_DEFAULTS[taskId] || c.get('model', 'gemma3:12b');
}

function getSensitiveWorkspaceOverrides() {
    const c = vscode.workspace.getConfiguration('sovereignAgent');
    return SENSITIVE_KEYS.filter(key => c.inspect(key)?.workspaceValue !== undefined);
}

function get() {
    const c = vscode.workspace.getConfiguration('sovereignAgent');
    return {
        provider:   c.get('provider',    'ollama'),
        baseUrl:    c.get('baseUrl',     'http://localhost:11434'),
        model:      c.get('model',       'gemma3:12b'),
        binaryPath: c.get('binaryPath',  'auto'),
        systemPrompt: c.get('systemPrompt', ''),
        taskModel: {
            docstring:  c.get('taskModel.docstring',  ''),
            tests:      c.get('taskModel.tests',      ''),
            typeHints:  c.get('taskModel.typeHints',  ''),
            commitMsg:  c.get('taskModel.commitMsg',  ''),
        },
        allowedTools: c.get('allowedTools', 'bash,read_file,write_file,list_files'),
        visionModel:  c.get('visionModel',  'qwen2.5vl:7b'),
        searchEngine: c.get('searchEngine', 'duckduckgo'),
    };
}

async function save(values) {
    const c = vscode.workspace.getConfiguration('sovereignAgent');
    // provider/baseUrl はグローバルに保存（機密設定）
    await c.update('provider',   values.provider,   vscode.ConfigurationTarget.Global);
    await c.update('baseUrl',    values.baseUrl,    vscode.ConfigurationTarget.Global);
    // その他はワークスペースに保存
    await c.update('model',      values.model,      vscode.ConfigurationTarget.Workspace);
    await c.update('systemPrompt', values.systemPrompt, vscode.ConfigurationTarget.Workspace);
    if (values.taskModel) {
        await c.update('taskModel.docstring',  values.taskModel.docstring  ?? '', vscode.ConfigurationTarget.Workspace);
        await c.update('taskModel.tests',      values.taskModel.tests      ?? '', vscode.ConfigurationTarget.Workspace);
        await c.update('taskModel.typeHints',  values.taskModel.typeHints  ?? '', vscode.ConfigurationTarget.Workspace);
        await c.update('taskModel.commitMsg',  values.taskModel.commitMsg  ?? '', vscode.ConfigurationTarget.Workspace);
    }
    if (values.allowedTools !== undefined)
        await c.update('allowedTools', values.allowedTools, vscode.ConfigurationTarget.Workspace);
    if (values.visionModel !== undefined)
        await c.update('visionModel',  values.visionModel,  vscode.ConfigurationTarget.Workspace);
    if (values.searchEngine !== undefined)
        await c.update('searchEngine', values.searchEngine, vscode.ConfigurationTarget.Global);
}

/** Ollama からモデル一覧を取得する */
async function fetchOllamaModels(baseUrl) {
    try {
        const https = baseUrl.startsWith('https') ? require('https') : require('http');
        const url = new URL('/api/tags', baseUrl);
        return await new Promise((resolve) => {
            https.get(url.toString(), (res) => {
                let data = '';
                res.on('data', d => data += d);
                res.on('end', () => {
                    try {
                        const json = JSON.parse(data);
                        resolve((json.models ?? []).map(m => m.name));
                    } catch { resolve([]); }
                });
            }).on('error', () => resolve([]));
        });
    } catch { return []; }
}

module.exports = { get, getTaskModel, getSensitiveWorkspaceOverrides, save, fetchOllamaModels };
