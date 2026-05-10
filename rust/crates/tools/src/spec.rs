use common::{ToolDef, ToolSpec};
use serde_json::json;

pub fn all_tool_defs() -> Vec<ToolDef> {
    vec![
        ToolDef {
            kind: "function".into(),
            function: ToolSpec {
                name: "bash".into(),
                description: "bash コマンドを実行して stdout/stderr を返す".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "実行するシェルコマンド" }
                    },
                    "required": ["command"]
                }),
            },
        },
        ToolDef {
            kind: "function".into(),
            function: ToolSpec {
                name: "read_file".into(),
                description: "ファイルの内容をテキストとして返す".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "読み込むファイルパス" }
                    },
                    "required": ["path"]
                }),
            },
        },
        ToolDef {
            kind: "function".into(),
            function: ToolSpec {
                name: "write_file".into(),
                description: "ファイルにテキストを書き込む（上書き）".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path":    { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }),
            },
        },
        ToolDef {
            kind: "function".into(),
            function: ToolSpec {
                name: "list_files".into(),
                description: "ディレクトリ内のファイル一覧を返す".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "対象ディレクトリ（省略時はカレント）" }
                    }
                }),
            },
        },
    ]
}
