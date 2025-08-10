use axum::body::Bytes;
use serde_json;

/// 通用MCP请求处理器，能够处理任何JSON格式
pub async fn handle_generic_mcp_request(body: Bytes) -> axum::Json<serde_json::Value> {
    // 尝试解析JSON
    let json_value: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(val) => val,
        Err(_) => {
            // 如果解析失败，返回JSON-RPC错误
            return axum::Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32700,
                    "message": "Parse error"
                }
            }));
        }
    };

    // 尝试提取基本字段
    let method = json_value
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");

    let id = json_value.get("id").cloned();
    let params = json_value.get("params").cloned();

    // 构造灵活的请求结构
    let request = crate::tools::tool_dto::FlexibleJsonRpcRequest {
        jsonrpc: json_value
            .get("jsonrpc")
            .and_then(|j| j.as_str())
            .unwrap_or("2.0")
            .to_string(),
        id: id.and_then(|i| match i {
            serde_json::Value::String(s) => Some(crate::tools::tool_dto::JsonRpcId::String(s)),
            serde_json::Value::Number(n) => {
                n.as_u64().map(crate::tools::tool_dto::JsonRpcId::Number)
            }
            serde_json::Value::Null => Some(crate::tools::tool_dto::JsonRpcId::Null),
            _ => None,
        }),
        method: method.to_string(),
        params,
    };

    handle_mcp_request_internal(request).await
}

/// 内部MCP请求处理逻辑
async fn handle_mcp_request_internal(
    request: crate::tools::tool_dto::FlexibleJsonRpcRequest<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    // 如果没有ID，这是一个通知，不需要响应
    let request_id = match &request.id {
        Some(id) => id.clone(),
        None => {
            // 对于通知，我们可以选择不响应或返回空响应
            return axum::Json(serde_json::json!({}));
        }
    };

    match request.method.as_str() {
        "initialize" => {
            let response = crate::tools::tool_dto::JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: serde_json::json!({
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "protocolVersion": "2025-06-18",
                    "serverInfo": {
                        "name": "local_mcp_servers",
                        "version": "0.1.0"
                    }
                }),
            };
            axum::Json(serde_json::to_value(&response).unwrap_or_default())
        }
        "tools/list" => {
            // 使用现有的工具列表处理逻辑
            let tools_request = crate::tools::tool_dto::JsonRpcRequest {
                jsonrpc: request.jsonrpc.clone(),
                id: request_id.clone(),
                method: request.method.clone(),
                params: request
                    .params
                    .as_ref()
                    .and_then(|p| serde_json::from_value(p.clone()).ok()),
            };
            let response = crate::tools::handler::handle_tools_list_internal(tools_request).await;
            axum::Json(serde_json::to_value(&response).unwrap_or_default())
        }
        _ => {
            let error_response = crate::tools::tool_dto::JsonRpcError {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                error: crate::tools::tool_dto::ErrorDetail {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                },
            };
            axum::Json(serde_json::to_value(&error_response).unwrap_or_default())
        }
    }
}
