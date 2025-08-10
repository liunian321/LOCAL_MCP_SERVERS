use crate::tools::{
    system_tool::handle_get_system_type, time_tool::handle_get_current_time, tool_dto::*,
};

/// 内部工具列表处理函数
pub async fn handle_tools_list_internal(
    request: JsonRpcRequest<ToolsListParams>,
) -> JsonRpcResponse<ToolsListResult> {
    let tools = vec![
        Tool {
            name: "get_system_type".to_string(),
            title: Some("系统类型信息".to_string()),
            description: "获取当前运行系统的类型信息，包括操作系统、架构".to_string(),
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
