'use strict';
const path = require('path');
const vscode = require('vscode');
const { workspaceRoot } = require('./workspace');

async function saveWebviewImage(image) {
    const root = workspaceRoot();
    const ext = safeImageExtension(image.mime || 'image/png');
    const fileName = 'webview-' + Date.now() + '.' + ext;
    const targetPath = path.join(root, '.sovereign', 'images', fileName);
    const bytes = Buffer.from(image.base64, 'base64');
    await vscode.workspace.fs.createDirectory(vscode.Uri.file(path.dirname(targetPath)));
    await vscode.workspace.fs.writeFile(vscode.Uri.file(targetPath), bytes);
    return path.relative(root, targetPath);
}

function safeImageExtension(mime) {
    if (mime === 'image/jpeg') return 'jpg';
    if (mime === 'image/webp') return 'webp';
    if (mime === 'image/gif') return 'gif';
    return 'png';
}

function looksLikeClipboardImageRequest(value) {
    return /(スクショ|スクリーンショット|クリップボード|画像|image|screenshot|clipboard)/i.test(value || '');
}

module.exports = { saveWebviewImage, looksLikeClipboardImageRequest };
