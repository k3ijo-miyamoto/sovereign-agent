'use strict';
const { spawn } = require('child_process');
const { resolveBinary } = require('./binary');

class Session {
    constructor(cfg, overrideModel, onEvent) {
        this._cfg = cfg;
        this._overrideModel = overrideModel || null;
        this._onEvent = onEvent;
        this._proc = null;
        this._lineBuf = '';
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

        this._proc = spawn(cmd, [...args, '--plain'], {
            env,
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
            console.error('[sovereign]', data.toString().trim());
        });

        this._proc.on('exit', code => {
            this._proc = null;
            if (code !== 0) {
                this._onEvent({ type: 'error', message: `プロセスが終了しました (code=${code})` });
            }
        });
    }

    send(text) {
        if (!this._proc) this.start();
        this._proc.stdin.write(text + '\n');
    }

    /** 現在のターンを中断する（プロセスは維持） */
    stop() {
        if (this._proc) {
            this._proc.stdin.write('/exit\n');
        }
    }

    dispose() {
        if (this._proc) {
            try { this._proc.stdin.end(); } catch { /* ignore */ }
            try { this._proc.kill(); } catch { /* ignore */ }
            this._proc = null;
        }
    }
}

module.exports = { Session };
