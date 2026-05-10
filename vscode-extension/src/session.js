'use strict';
const { spawn } = require('child_process');
const { resolveBinary } = require('./binary');
const { workspaceRoot } = require('./workspace');

class Session {
    constructor(cfg, overrideModel, onEvent) {
        this._cfg = cfg;
        this._overrideModel = overrideModel || null;
        this._onEvent = onEvent;
        this._proc = null;
        this._lineBuf = '';
        this._disposed = false;
    }

    start() {
        if (this._proc) return;
        const { cmd, args } = resolveBinary(this._cfg.binaryPath);
        const model = this._overrideModel || this._cfg.model;
        const env = {
            ...process.env,
            SOVEREIGN_MODEL:    model,
            SOVEREIGN_PROVIDER: this._cfg.provider,
            OLLAMA_BASE_URL:    this._cfg.baseUrl,
        };

        const cwd = workspaceRoot();
        const cliArgs = [...args, '--cwd', cwd, '--plain'];
        if (this._cfg.allowedTools) cliArgs.push('--allowed-tools', this._cfg.allowedTools);
        if (this._cfg.visionModel)  cliArgs.push('--vision-model',  this._cfg.visionModel);

        this._proc = spawn(cmd, cliArgs, {
            env,
            cwd,
            stdio: ['pipe', 'pipe', 'pipe'],
        });

        this._proc.stdout.on('data', data => {
            this._lineBuf += data.toString();
            let nl;
            while ((nl = this._lineBuf.indexOf('\n')) !== -1) {
                const line = this._lineBuf.slice(0, nl).trim();
                this._lineBuf = this._lineBuf.slice(nl + 1);
                if (!line) continue;
                try { this._onEvent(JSON.parse(line)); } catch { /* 不正行は無視 */ }
            }
        });

        this._proc.stderr.on('data', data => {
            const text = data.toString().trim();
            if (text) this._onEvent({ type: 'error', message: text });
        });

        this._proc.on('exit', code => {
            this._proc = null;
            if (!this._disposed && code !== 0) {
                this._onEvent({ type: 'error', message: `プロセスが終了しました (code=${code})` });
            }
        });
    }

    send(text) {
        if (!this._proc) this.start();
        this._proc.stdin.write(text + '\n');
    }

    sendJson(obj) {
        if (!this._proc) this.start();
        this._proc.stdin.write(JSON.stringify(obj) + '\n');
    }

    /** 現在のターンを中断する（プロセスは維持） */
    stop() {
        if (this._proc) {
            this._proc.stdin.write('/exit\n');
        }
    }

    dispose() {
        this._disposed = true;
        if (this._proc) {
            try { this._proc.stdin.end(); } catch { /* ignore */ }
            try { this._proc.kill(); } catch { /* ignore */ }
            this._proc = null;
        }
    }
}

module.exports = { Session };
