mod bash;
mod fs;
mod spec;
pub mod mcp;

pub use mcp::{McpServer, McpToolDef, load_config as load_mcp_config};
pub use spec::all_tool_defs;

use agent::{ToolExecutor, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub struct LocalExecutor;

#[async_trait]
impl ToolExecutor for LocalExecutor {
    async fn execute(&self, name: &str, call_id: &str, arguments: &Value) -> ToolResult {
        let result = match name {
            "bash"        => bash::run(arguments).await,
            "read_file"   => fs::read_file(arguments),
            "write_file"  => fs::write_file(arguments),
            "list_files"  => fs::list_files(arguments),
            "grep_search" => fs::grep_search(arguments),
            "glob_search" => fs::glob_search(arguments),
            "edit_file"   => fs::edit_file(arguments),
            other         => Err(anyhow::anyhow!("未知のツール: {other}")),
        };
        match result {
            Ok(output) => ToolResult::ok(call_id, output),
            Err(e)     => ToolResult::err(call_id, e.to_string()),
        }
    }
}

/// ローカルツール + MCP サーバーを束ねた実行器
/// ツール名でルーティング: MCP に登録されていなければ LocalExecutor に委譲
pub struct CombinedExecutor {
    routes: HashMap<String, Arc<McpServer>>,
}

impl CombinedExecutor {
    pub fn new(servers: Vec<Arc<McpServer>>) -> Self {
        let mut routes = HashMap::new();
        for server in servers {
            for tool in &server.tools {
                routes.insert(tool.name.clone(), Arc::clone(&server));
            }
        }
        Self { routes }
    }
}

#[async_trait]
impl ToolExecutor for CombinedExecutor {
    async fn execute(&self, name: &str, call_id: &str, arguments: &Value) -> ToolResult {
        if let Some(server) = self.routes.get(name) {
            match server.call(name, arguments).await {
                Ok(out) => ToolResult::ok(call_id, out),
                Err(e)  => ToolResult::err(call_id, e.to_string()),
            }
        } else {
            LocalExecutor.execute(name, call_id, arguments).await
        }
    }
}
