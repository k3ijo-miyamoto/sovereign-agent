mod bash;
mod fs;
mod spec;

pub use spec::all_tool_defs;

use agent::{ToolExecutor, ToolResult};
use async_trait::async_trait;
use serde_json::Value;

pub struct LocalExecutor;

#[async_trait]
impl ToolExecutor for LocalExecutor {
    async fn execute(&self, name: &str, call_id: &str, arguments: &Value) -> ToolResult {
        let result = match name {
            "bash"       => bash::run(arguments).await,
            "read_file"  => fs::read_file(arguments),
            "write_file" => fs::write_file(arguments),
            "list_files" => fs::list_files(arguments),
            other        => Err(anyhow::anyhow!("未知のツール: {other}")),
        };
        match result {
            Ok(output) => ToolResult::ok(call_id, output),
            Err(e)     => ToolResult::err(call_id, e.to_string()),
        }
    }
}
