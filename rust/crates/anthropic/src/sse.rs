/// SSE ストリームを行単位でパースして SseEvent を返すイテレータ
use anyhow::{Context, Result};

use crate::wire::SseEvent;

pub fn parse_sse_line(line: &str) -> Option<Result<SseEvent>> {
    let data = line.strip_prefix("data: ")?;
    if data == "[DONE]" {
        return None;
    }
    Some(serde_json::from_str(data).context("SSE イベントのパース失敗"))
}
