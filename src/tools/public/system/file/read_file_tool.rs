use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

use crate::tools::tool_dto::{ToolCallResult, ToolContent};

#[derive(Serialize, Deserialize)]
struct ReadFile {
    file_path: String,
    contents: Option<String>,
}

async fn read_file_tool(file_path: String) -> Result<String, std::io::Error> {
    let file = File::open(&file_path).await?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).await?;
    Ok(contents)
}

pub async fn handle_read_file_tool(args_json: Option<serde_json::Value>) -> ToolCallResult {
    let args = match serde_json::from_value::<ReadFile>(match args_json {
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
    }) {
        Ok(args) => args,
        Err(e) => {
            return ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(e.to_string()),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(true),
                structured_content: Some(
                    serde_json::to_value(String::from("查询路径有误")).unwrap(),
                ),
            };
        }
    };

    let file_path = args.file_path.clone();
    
    match read_file_tool(file_path.clone()).await {
        Ok(contents) => {
            ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(contents.clone()),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(false),
                structured_content: Some(
                    serde_json::to_value(serde_json::json!({
                        "file_path": file_path,
                        "contents": contents
                    }))
                    .unwrap(),
                ),
            }
        }
        Err(e) => {
            ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(format!("读取文件失败: {}", e)),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(true),
                structured_content: Some(
                    serde_json::to_value(format!("读取文件失败: {}", e)).unwrap(),
                ),
            }
        }
    }
}
