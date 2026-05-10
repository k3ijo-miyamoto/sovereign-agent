'use strict';

function getWebviewHtml() {
    return /* html */`<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Sovereign Agent</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: var(--vscode-font-family);
  font-size: var(--vscode-font-size);
  color: var(--vscode-foreground);
  background: var(--vscode-editor-background);
  display: flex; flex-direction: column; height: 100vh; overflow: hidden;
}

/* ── Settings panel ─────────────────────────────── */
#settings-panel {
  border-bottom: 1px solid var(--vscode-panel-border);
  background: var(--vscode-sideBar-background);
}
#settings-toggle {
  width: 100%; padding: 5px 10px; text-align: left; cursor: pointer;
  background: none; border: none; color: var(--vscode-foreground);
  font: inherit; font-weight: bold; font-size: 0.9em;
  display: flex; align-items: center; gap: 6px;
}
#settings-toggle:hover { background: var(--vscode-list-hoverBackground); }
#settings-body { display: none; padding: 8px 10px; }
#settings-body.open { display: block; }

.settings-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: 6px 12px;
  margin-bottom: 8px;
}
.settings-grid label, .settings-full label {
  display: flex; flex-direction: column; gap: 2px;
  font-size: 0.82em; color: var(--vscode-descriptionForeground);
}
.settings-grid input, .settings-grid select,
.settings-full input, .settings-full select, .settings-full textarea {
  background: var(--vscode-input-background);
  color: var(--vscode-input-foreground);
  border: 1px solid var(--vscode-input-border);
  border-radius: 3px; padding: 3px 6px; font: inherit; font-size: 0.9em;
  width: 100%;
}
.settings-full { margin-bottom: 6px; }
.settings-full textarea { resize: vertical; min-height: 52px; }

.task-models {
  display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 6px;
  margin-bottom: 8px;
}
.task-models label {
  display: flex; flex-direction: column; gap: 2px;
  font-size: 0.8em; color: var(--vscode-descriptionForeground);
}
.task-models select {
  background: var(--vscode-input-background);
  color: var(--vscode-input-foreground);
  border: 1px solid var(--vscode-input-border);
  border-radius: 3px; padding: 3px 4px; font: inherit; font-size: 0.82em;
}

#apply-btn {
  background: var(--vscode-button-background);
  color: var(--vscode-button-foreground);
  border: none; border-radius: 3px; padding: 4px 12px;
  cursor: pointer; font: inherit; font-size: 0.85em;
}
#apply-btn:hover { background: var(--vscode-button-hoverBackground); }

/* ── Quick tasks ────────────────────────────────── */
#quick-tasks {
  padding: 5px 10px;
  border-bottom: 1px solid var(--vscode-panel-border);
  display: flex; align-items: center; gap: 6px; flex-wrap: wrap;
}
#quick-tasks span { font-size: 0.82em; color: var(--vscode-descriptionForeground); }
#model-group {
  margin-left: auto; display: flex; align-items: center; gap: 4px; flex-shrink: 0;
}
#model-group label { font-size: 0.82em; color: var(--vscode-descriptionForeground); white-space: nowrap; }
#main-model-select {
  background: var(--vscode-input-background);
  color: var(--vscode-input-foreground);
  border: 1px solid var(--vscode-input-border);
  border-radius: 3px; padding: 2px 4px; font: inherit; font-size: 0.82em;
  max-width: 160px;
}
.task-btn {
  padding: 2px 10px; border-radius: 3px; border: 1px solid var(--vscode-button-border, #555);
  background: var(--vscode-button-secondaryBackground);
  color: var(--vscode-button-secondaryForeground);
  cursor: pointer; font: inherit; font-size: 0.82em; white-space: nowrap;
}
.task-btn:hover { background: var(--vscode-button-secondaryHoverBackground); }
.task-btn:disabled { opacity: 0.45; cursor: not-allowed; }

/* ── Messages ───────────────────────────────────── */
#messages {
  flex: 1; overflow-y: auto; padding: 10px;
  display: flex; flex-direction: column; gap: 7px;
}
.msg {
  max-width: 88%; padding: 7px 11px; border-radius: 7px;
  white-space: pre-wrap; word-break: break-word; line-height: 1.5;
}
.msg.user {
  align-self: flex-end;
  background: var(--vscode-button-background);
  color: var(--vscode-button-foreground);
}
.msg.assistant { align-self: flex-start; background: var(--vscode-editor-inactiveSelectionBackground); }
.msg.tool {
  align-self: flex-start; font-size: 0.82em;
  color: var(--vscode-descriptionForeground); background: transparent; padding: 1px 6px;
}
.msg.system {
  align-self: center; font-size: 0.8em; font-style: italic;
  color: var(--vscode-descriptionForeground); background: transparent;
}
.msg.error { align-self: flex-start; background: var(--vscode-inputValidation-errorBackground); }
.thinking::after { content: '●'; animation: blink 1s step-start infinite; }
@keyframes blink { 50% { opacity: 0; } }

/* ── Image preview ──────────────────────────────── */
#img-preview {
  display: none; align-items: center; gap: 6px;
  padding: 4px 8px; border-top: 1px solid var(--vscode-panel-border);
  background: var(--vscode-sideBar-background);
}
#img-thumb {
  height: 48px; border-radius: 3px;
  border: 1px solid var(--vscode-input-border);
}
#img-preview span { font-size: 0.8em; color: var(--vscode-descriptionForeground); flex: 1; }
#img-clear {
  background: none; border: none; cursor: pointer; font-size: 1em;
  color: var(--vscode-foreground); opacity: 0.6; padding: 0 4px;
}
#img-clear:hover { opacity: 1; }

/* ── Input row ──────────────────────────────────── */
#input-row {
  display: flex; gap: 5px; padding: 7px 8px;
  border-top: 1px solid var(--vscode-panel-border);
}
#input {
  flex: 1; resize: none;
  background: var(--vscode-input-background);
  color: var(--vscode-input-foreground);
  border: 1px solid var(--vscode-input-border);
  border-radius: 4px; padding: 5px 8px;
  font: inherit; min-height: 34px; max-height: 120px;
}
#input:focus { outline: 1px solid var(--vscode-focusBorder); }
#input.drag-over { outline: 2px solid var(--vscode-focusBorder); }
#send-btn {
  background: var(--vscode-button-background);
  color: var(--vscode-button-foreground);
  border: none; border-radius: 4px; padding: 0 14px;
  cursor: pointer; font: inherit;
}
#send-btn:hover { background: var(--vscode-button-hoverBackground); }
#send-btn:disabled { opacity: 0.5; cursor: not-allowed; }

/* ── Action buttons ─────────────────────────────── */
#action-row {
  display: flex; gap: 5px; padding: 4px 8px 6px;
  border-top: 1px solid var(--vscode-panel-border);
  flex-wrap: wrap;
}
.action-btn {
  padding: 2px 9px; border-radius: 3px;
  border: 1px solid var(--vscode-button-border, #555);
  background: var(--vscode-button-secondaryBackground);
  color: var(--vscode-button-secondaryForeground);
  cursor: pointer; font: inherit; font-size: 0.82em;
}
.action-btn:hover { background: var(--vscode-button-secondaryHoverBackground); }
</style>
</head>
<body>

<!-- ── Settings panel ── -->
<div id="settings-panel">
  <button id="settings-toggle">▼ Settings</button>
  <div id="settings-body">
    <div class="settings-grid">
      <label>Provider
        <select id="s-provider">
          <option value="ollama">ollama</option>
          <option value="anthropic">anthropic</option>
        </select>
      </label>
      <label>Base URL
        <input id="s-base-url" type="text">
      </label>
      <label>Text model
        <select id="s-model"><option value="">— loading —</option></select>
      </label>
      <label>Binary path
        <input id="s-binary" type="text" placeholder="auto">
      </label>
    </div>

    <div class="settings-full">
      <label>System prompt (appended to default)
        <textarea id="s-system" rows="3"></textarea>
      </label>
    </div>

    <div class="settings-grid">
      <label>Vision model
        <input id="s-vision" type="text" placeholder="例: qwen2.5vl:7b">
      </label>
      <label>Search engine
        <select id="s-search">
          <option value="duckduckgo">DuckDuckGo</option>
          <option value="google">Google</option>
          <option value="bing">Bing</option>
        </select>
      </label>
    </div>

    <div class="settings-full">
      <label>Allowed tools (カンマ区切り)
        <input id="s-tools" type="text" placeholder="bash,read_file,write_file,list_files">
      </label>
    </div>

    <div style="font-size:0.82em;color:var(--vscode-descriptionForeground);margin-bottom:4px;">
      タスク別モデル（空欄 = CLAUDE.md デフォルトを使用）
    </div>
    <div class="task-models">
      <label>📄 Docstring  <select id="tm-docstring"></select></label>
      <label>🧪 Tests      <select id="tm-tests"></select></label>
      <label>🏷 Type hints <select id="tm-typeHints"></select></label>
      <label>💬 Commit msg <select id="tm-commitMsg"></select></label>
    </div>

    <button id="apply-btn">Apply and Restart CLI</button>
  </div>
</div>

<!-- ── Quick tasks ── -->
<div id="quick-tasks">
  <span>Quick tasks:</span>
  <button class="task-btn" data-task="docstring">📄 Docstring</button>
  <button class="task-btn" data-task="tests">🧪 Tests</button>
  <button class="task-btn" data-task="typeHints">🏷 Type hints</button>
  <button class="task-btn" data-task="commitMsg">💬 Commit msg</button>
  <div id="model-group">
    <label for="main-model-select">Model:</label>
    <select id="main-model-select"><option value="">— loading —</option></select>
  </div>
</div>

<!-- ── Messages ── -->
<div id="messages"></div>

<!-- ── Image preview ── -->
<div id="img-preview">
  <img id="img-thumb" alt="preview">
  <span id="img-label">画像を添付中</span>
  <button id="img-clear" title="キャンセル">✕</button>
</div>

<!-- ── Input ── -->
<div id="input-row">
  <textarea id="input" placeholder="メッセージを入力… (Shift+Enter で改行 / 画像はペースト可)" rows="1"></textarea>
  <button id="send-btn">送信</button>
</div>

<!-- ── Action buttons ── -->
<div id="action-row">
  <button class="action-btn" id="btn-restart">Restart CLI</button>
  <button class="action-btn" id="btn-stop">Stop</button>
  <button class="action-btn" id="btn-clear">Clear</button>
  <button class="action-btn" id="btn-export">Export Transcript</button>
</div>

<script>
const vscode = acquireVsCodeApi();

// ── state ──────────────────────────────────────────
let busy = false;
let currentAssistantEl = null;
let availableModels = [];
let pendingImage = null;
const taskModelKeys = ['docstring', 'tests', 'typeHints', 'commitMsg'];

// ── DOM refs ───────────────────────────────────────
const messagesEl   = document.getElementById('messages');
const inputEl      = document.getElementById('input');
const sendBtn      = document.getElementById('send-btn');
const settingsBody = document.getElementById('settings-body');
const imgPreview   = document.getElementById('img-preview');
const imgThumb     = document.getElementById('img-thumb');
const imgLabel     = document.getElementById('img-label');

// ── helpers ────────────────────────────────────────
function scrollBottom() { messagesEl.scrollTop = messagesEl.scrollHeight; }

function addMsg(cls, text) {
    const el = document.createElement('div');
    el.className = 'msg ' + cls;
    el.textContent = text;
    messagesEl.appendChild(el);
    scrollBottom();
    return el;
}

function setBusy(val) {
    busy = val;
    sendBtn.disabled = val;
    inputEl.disabled = val;
    document.querySelectorAll('.task-btn').forEach(b => b.disabled = val);
}

// ── settings panel ─────────────────────────────────
document.getElementById('settings-toggle').addEventListener('click', () => {
    const open = settingsBody.classList.toggle('open');
    document.getElementById('settings-toggle').textContent = (open ? '▲' : '▼') + ' Settings';
    if (open) vscode.postMessage({ type: 'get_settings' });
});

function populateModelSelect(sel, models, current) {
    const defaultOpt = '<option value="">— デフォルトモデルを使用 —</option>';
    sel.innerHTML = (sel.id === 's-model' ? '' : defaultOpt) +
        models.map(m => '<option value="' + m + '"' + (m === current ? ' selected' : '') + '>' + m + '</option>').join('');
    if (sel.id !== 's-model' && !current) sel.value = '';
}

function applySettings(cfg, models) {
    availableModels = models;
    document.getElementById('s-provider').value = cfg.provider;
    document.getElementById('s-base-url').value  = cfg.baseUrl;
    document.getElementById('s-binary').value    = cfg.binaryPath;
    document.getElementById('s-system').value    = cfg.systemPrompt;
    document.getElementById('s-vision').value    = cfg.visionModel  || '';
    document.getElementById('s-search').value    = cfg.searchEngine || 'duckduckgo';
    document.getElementById('s-tools').value     = cfg.allowedTools || 'bash,read_file,write_file,list_files';

    populateModelSelect(document.getElementById('s-model'), models, cfg.model);
    taskModelKeys.forEach(k => {
        populateModelSelect(
            document.getElementById('tm-' + k), models,
            cfg.taskModel ? (cfg.taskModel[k] || '') : ''
        );
    });

    // populate the always-visible model selector
    const mainSel = document.getElementById('main-model-select');
    mainSel.innerHTML = models.length
        ? models.map(m => '<option value="' + m + '"' + (m === cfg.model ? ' selected' : '') + '>' + m + '</option>').join('')
        : '<option value="' + cfg.model + '" selected>' + cfg.model + '</option>';
}

document.getElementById('apply-btn').addEventListener('click', () => {
    const taskModel = {};
    taskModelKeys.forEach(k => {
        taskModel[k] = document.getElementById('tm-' + k).value;
    });
    vscode.postMessage({
        type: 'save_settings',
        settings: {
            provider:     document.getElementById('s-provider').value,
            baseUrl:      document.getElementById('s-base-url').value,
            model:        document.getElementById('s-model').value,
            binaryPath:   document.getElementById('s-binary').value,
            systemPrompt: document.getElementById('s-system').value,
            visionModel:  document.getElementById('s-vision').value,
            searchEngine: document.getElementById('s-search').value,
            allowedTools: document.getElementById('s-tools').value,
            taskModel,
        }
    });
    settingsBody.classList.remove('open');
    document.getElementById('settings-toggle').textContent = '▼ Settings';
});

// ── model selector ─────────────────────────────────
document.getElementById('main-model-select').addEventListener('change', function() {
    if (!this.value) return;
    vscode.postMessage({ type: 'set_model', model: this.value });
    addMsg('system', '— モデルを ' + this.value + ' に変更して再起動しました —');
});

// ── quick task buttons ─────────────────────────────
document.querySelectorAll('.task-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        vscode.postMessage({ type: 'task', task: btn.dataset.task });
    });
});

// ── image paste / drop ─────────────────────────────
function attachPendingImage(file) {
    const reader = new FileReader();
    reader.onload = evt => {
        const dataUrl = evt.target.result;
        const parts = dataUrl.split(',');
        const mimeMatch = parts[0].match(/:(.*?);/);
        const mime = mimeMatch ? mimeMatch[1] : 'image/png';
        pendingImage = { base64: parts[1], mime };
        imgThumb.src = dataUrl;
        imgLabel.textContent = file.name || '画像を添付中';
        imgPreview.style.display = 'flex';
    };
    reader.readAsDataURL(file);
}

inputEl.addEventListener('paste', e => {
    const items = Array.from(e.clipboardData ? e.clipboardData.items : []);
    const imageItem = items.find(i => i.type.startsWith('image/'));
    if (!imageItem) return;
    e.preventDefault();
    attachPendingImage(imageItem.getAsFile());
});

inputEl.addEventListener('dragover', e => {
    if (Array.from(e.dataTransfer.types).includes('Files')) {
        e.preventDefault();
        inputEl.classList.add('drag-over');
    }
});
inputEl.addEventListener('dragleave', () => inputEl.classList.remove('drag-over'));
inputEl.addEventListener('drop', e => {
    inputEl.classList.remove('drag-over');
    const file = e.dataTransfer.files[0];
    if (file && file.type.startsWith('image/')) {
        e.preventDefault();
        attachPendingImage(file);
    }
});

document.getElementById('img-clear').addEventListener('click', () => {
    pendingImage = null;
    imgPreview.style.display = 'none';
    imgThumb.src = '';
});

// ── chat input ─────────────────────────────────────
function send() {
    const text = inputEl.value.trim();
    if ((!text && !pendingImage) || busy) return;
    inputEl.value = '';
    inputEl.style.height = 'auto';

    if (pendingImage) {
        vscode.postMessage({ type: 'image', images: [pendingImage], text });
        pendingImage = null;
        imgPreview.style.display = 'none';
        imgThumb.src = '';
    } else {
        vscode.postMessage({ type: 'send', text });
    }
}
sendBtn.addEventListener('click', send);
inputEl.addEventListener('keydown', e => {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); }
});
inputEl.addEventListener('input', () => {
    inputEl.style.height = 'auto';
    inputEl.style.height = Math.min(inputEl.scrollHeight, 120) + 'px';
});

// ── action buttons ─────────────────────────────────
document.getElementById('btn-restart').addEventListener('click', () => {
    vscode.postMessage({ type: 'restart' });
});
document.getElementById('btn-stop').addEventListener('click', () => {
    vscode.postMessage({ type: 'stop' });
});
document.getElementById('btn-clear').addEventListener('click', () => {
    vscode.postMessage({ type: 'clear' });
});
document.getElementById('btn-export').addEventListener('click', () => {
    vscode.postMessage({ type: 'export' });
});

// ── messages from extension ────────────────────────
window.addEventListener('message', e => {
    const msg = e.data;
    switch (msg.type) {
        case 'user':
            currentAssistantEl = null;
            addMsg('user', msg.text);
            setBusy(true);
            break;
        case 'thinking':
            currentAssistantEl = addMsg('assistant thinking', '');
            setBusy(true);
            break;
        case 'text_delta':
            if (!currentAssistantEl) currentAssistantEl = addMsg('assistant', '');
            currentAssistantEl.classList.remove('thinking');
            currentAssistantEl.textContent += msg.delta;
            scrollBottom();
            break;
        case 'tool_start':
            addMsg('tool', '⚙ ' + msg.name + '…');
            break;
        case 'tool_done':
            addMsg('tool', (msg.ok ? '✓' : '✗') + ' ' + msg.name);
            break;
        case 'ready':
            currentAssistantEl = null;
            setBusy(false);
            inputEl.focus();
            break;
        case 'error':
            if (currentAssistantEl) {
                currentAssistantEl.classList.remove('thinking');
                currentAssistantEl = null;
            }
            addMsg('error', '⚠ ' + msg.message);
            setBusy(false);
            break;
        case 'system':
            addMsg('system', msg.text);
            break;
        case 'clear':
            messagesEl.innerHTML = '';
            currentAssistantEl = null;
            break;
        case 'settings':
            applySettings(msg.settings, msg.models);
            break;
    }
});

inputEl.focus();
// fetch models on load so the selector is populated immediately
vscode.postMessage({ type: 'get_settings' });
</script>
</body>
</html>`;
}

module.exports = { getWebviewHtml };
