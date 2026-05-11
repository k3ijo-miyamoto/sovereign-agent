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
        ToolDef {
            kind: "function".into(),
            function: ToolSpec {
                name: "grep_search".into(),
                description: "正規表現パターンでファイルを再帰検索し、マッチした行を返す".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern":          { "type": "string", "description": "検索する正規表現" },
                        "path":             { "type": "string", "description": "検索対象ディレクトリ（省略時はカレント）" },
                        "include":          { "type": "string", "description": "対象ファイルの glob パターン（例: *.rs）" },
                        "case_insensitive": { "type": "boolean", "description": "大文字小文字を無視するか（デフォルト false）" }
                    },
                    "required": ["pattern"]
                }),
            },
        },
        ToolDef {
            kind: "function".into(),
            function: ToolSpec {
                name: "glob_search".into(),
                description: "ファイル名の glob パターンでファイルを再帰検索し、パス一覧を返す".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "ファイル名の glob パターン（例: *.rs, test_*.py）" },
                        "path":    { "type": "string", "description": "検索対象ディレクトリ（省略時はカレント）" }
                    },
                    "required": ["pattern"]
                }),
            },
        },
        ToolDef {
            kind: "function".into(),
            function: ToolSpec {
                name: "edit_file".into(),
                description: "ファイル内の文字列を1箇所だけ置換する。old_string はファイル内で一意である必要がある".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path":       { "type": "string", "description": "編集対象のファイルパス" },
                        "old_string": { "type": "string", "description": "置換前の文字列（ファイル内で一意であること）" },
                        "new_string": { "type": "string", "description": "置換後の文字列" }
                    },
                    "required": ["path", "old_string", "new_string"]
                }),
            },
        },
    ]
}
