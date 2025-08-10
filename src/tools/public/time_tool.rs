use chrono::Local;

use crate::tools::tool_dto::{ToolCallResult, ToolContent};

fn get_current_time() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn handle_get_current_time(_args: Option<serde_json::Value>) -> ToolCallResult {
    let result_text = get_current_time();
    ToolCallResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: Some(result_text.clone()),
            data: None,
            mime_type: None,
            annotations: None,
        }],
        is_error: Some(false),
        structured_content: Some(serde_json::json!({
            "timestamp": result_text,
            "format": "YYYY-MM-DD HH:MM:SS"
        })),
    }
}
