use serde::{Deserialize, Serialize};
use std::env;

use crate::tools::tool_dto::{ToolCallResult, ToolContent};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
}

fn get_system_type() -> SystemInfo {
    SystemInfo {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
    }
}

/**
 * 获取系统类型
 */
pub fn handle_get_system_type(_args: Option<serde_json::Value>) -> ToolCallResult {
    let system_info = get_system_type();
    let result_text = serde_json::to_string_pretty(&system_info)
        .unwrap_or_else(|_| "Failed to serialize system information".to_string());

    ToolCallResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: Some(result_text),
            data: None,
            mime_type: None,
            annotations: None,
        }],
        is_error: Some(false),
        structured_content: Some(serde_json::to_value(system_info).unwrap_or_default()),
    }
}
