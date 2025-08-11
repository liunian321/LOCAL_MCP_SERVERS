use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::fs;

use crate::tools::tool_dto::{ToolCallResult, ToolContent};

#[derive(Serialize, Deserialize)]
struct ListFiles {
    dir_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct FileInfo {
    name: String,
    size: u64,
    modified: String,
    is_dir: bool,
}

async fn list_files_tool(dir_path: String) -> Vec<FileInfo> {
    let mut files = fs::read_dir(dir_path).await.unwrap();
    let mut file_infos = Vec::new();

    while let Some(entry) = files.next_entry().await.unwrap() {
        let path = entry.path();
        let metadata = entry.metadata().await.unwrap();

        let modified = metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file_info = FileInfo {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            size: metadata.len(),
            modified: format!("{}", modified),
            is_dir: metadata.is_dir(),
        };

        file_infos.push(file_info);
    }

    file_infos
}

pub async fn handle_list_files_tool(args_json: Option<serde_json::Value>) -> ToolCallResult {
    let args = serde_json::from_value::<ListFiles>(match args_json {
        Some(args) => args,
        None => {
            return ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(String::from("参数传递错误")),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(true),
                structured_content: Some(
                    serde_json::to_value(String::from("参数传递错误")).unwrap(),
                ),
            };
        }
    })
    .unwrap();

    let files = list_files_tool(args.dir_path).await;

    let result_object = serde_json::json!({
        "files": files
    });

    return ToolCallResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: Some(serde_json::to_string(&files).unwrap()),
            data: None,
            mime_type: None,
            annotations: None,
        }],
        is_error: Some(false),
        structured_content: Some(result_object),
    };
}
