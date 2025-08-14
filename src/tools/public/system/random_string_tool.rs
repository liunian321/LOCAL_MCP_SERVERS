use crate::tools::tool_dto::{ToolCallResult, ToolContent};
use rand::Rng;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RandomStringParams {
    #[serde(default = "default_length")]
    pub length: usize,
    #[serde(default = "default_include_special")]
    pub include_special: bool,
}

impl Default for RandomStringParams {
    fn default() -> Self {
        Self {
            length: default_length(),
            include_special: default_include_special(),
        }
    }
}

fn default_length() -> usize {
    16
}

fn default_include_special() -> bool {
    false
}

fn generate_random_string(length: usize, include_special: Option<bool>) -> String {
    let mut rng = rand::thread_rng();

    // 验证长度范围
    let length = length.clamp(1, 4096);

    // 定义字符集
    let mut chars = Vec::new();

    // 大写字母 A-Z
    chars.extend(b'A'..=b'Z');
    // 小写字母 a-z
    chars.extend(b'a'..=b'z');
    // 数字 0-9
    chars.extend(b'0'..=b'9');

    // 可选的特殊字符
    if include_special.unwrap_or(false) {
        chars.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    // 生成随机字符串
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx] as char
        })
        .collect()
}

pub fn handle_random_string_tool(args_json: Option<serde_json::Value>) -> ToolCallResult {
    let params = serde_json::from_value::<RandomStringParams>(match args_json {
        Some(args) => args,
        None => {
            return ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some("缺少参数".to_string()),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(true),
                structured_content: Some(serde_json::to_value(String::from("缺少参数")).unwrap()),
            };
        }
    })
    .unwrap();

    // 验证参数
    if params.length < 1 || params.length > 4096 {
        return ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: Some("字符串长度必须为1到4096!".to_string()),
                data: None,
                mime_type: None,
                annotations: None,
            }],
            is_error: Some(true),
            structured_content: None,
        };
    }

    let random_string = generate_random_string(params.length, Some(params.include_special));

    ToolCallResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: Some(random_string),
            data: None,
            mime_type: None,
            annotations: None,
        }],
        is_error: Some(false),
        structured_content: None,
    }
}
