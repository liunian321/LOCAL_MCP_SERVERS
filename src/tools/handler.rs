use crate::tools::{
    public::{
        file::{list_files_tool::handle_list_files_tool, read_file_tool::handle_read_file_tool},
        network::{ping_tool::handle_ping_tool, read_ip_tool::handle_read_ip_tool},
        system_tool::handle_get_system_type,
        time_tool::handle_get_current_time,
    },
    tool_dto::*,
};

/// 内部工具列表处理函数
pub async fn handle_tools_list_internal(
    request: JsonRpcRequest<ToolsListParams>,
) -> JsonRpcResponse<ToolsListResult> {
    let tools = vec![
        Tool {
            name: "get_system_type".to_string(),
            title: Some("系统类型信息".to_string()),
            description: "获取当前运行系统的类型信息,包括操作系统、架构".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({})),
                required: None,
            },
            output_schema: None,
            annotations: None,
        },
        Tool {
            name: "get_current_time".to_string(),
            title: Some("当前时间".to_string()),
            description: "获取当前时间".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({})),
                required: None,
            },
            output_schema: None,
            annotations: None,
        },
        Tool {
            name: "ping".to_string(),
            title: Some("Ping".to_string()),
            description: "Ping 工具,测试网络连通性".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({
                    "target": {
                        "type": "string",
                        "description": "要ping的地址,支持URL,IP:端口,IP地址,域名"
                    }
                })),
                required: Some(vec!["target".to_string()]),
            },
            output_schema: None,
            annotations: None,
        },
        Tool {
            name: "read ip".to_string(),
            title: Some("查询IP".to_string()),
            description: "查询域名解析IP及延迟；不传参数时返回本机公网IP".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({
                    "domain": {"type": "string", "description": "要解析的域名，可选"},
                    "dns": {"type": "string", "description": "自定义DNS服务器，支持 ip 或 ip:port，可选"},
                    "port": {"type": "number", "description": "用于测延迟的端口，默认80，可选"}
                })),
                required: None,
            },
            output_schema: None,
            annotations: None,
        },
        Tool {
            name: "cat file".to_string(),
            title: Some("读取文件".to_string()),
            description: "读取文件内容".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({
                    "file_path": {
                        "type": "string",
                        "description": "要读取的文件路径"
                    }
                })),
                required: Some(vec!["file_path".to_string()]),
            },
            output_schema: None,
            annotations: None,
        },
        Tool {
            name: "list files".to_string(),
            title: Some("列出文件".to_string()),
            description: "列出文件".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({
                    "dir_path": {
                        "type": "string",
                        "description": "要列出的文件夹路径"
                    }
                })),
                required: Some(vec!["dir_path".to_string()]),
            },
            output_schema: None,
            annotations: None,
        },
    ];

    let result = ToolsListResult {
        tools,
        next_cursor: None,
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result,
    }
}

/// 内部工具调用处理函数
pub async fn handle_tool_call_internal(
    request: JsonRpcRequest<ToolCallParams>,
) -> Result<JsonRpcResponse<ToolCallResult>, JsonRpcError> {
    let params = request.params.unwrap_or(ToolCallParams {
        name: "".to_string(),
        arguments: None,
    });

    let result = match params.name.as_str() {
        "get_system_type" => handle_get_system_type(params.arguments),
        "get_current_time" => handle_get_current_time(params.arguments),
        "ping" => handle_ping_tool(params.arguments),
        "read ip" => handle_read_ip_tool(params.arguments).await,
        "cat file" => handle_read_file_tool(params.arguments).await,
        "list files" => handle_list_files_tool(params.arguments).await,
        _ => {
            return Err(JsonRpcError {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                error: ErrorDetail {
                    code: -32602,
                    message: format!("Unknown tool: {}", params.name),
                    data: None,
                },
            });
        }
    };

    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result,
    })
}
