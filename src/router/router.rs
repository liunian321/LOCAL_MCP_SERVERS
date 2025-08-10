use axum::{
    Router,
    body::Bytes,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::{get, post},
};
use futures::stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;

pub fn init_bind_router(app: Router) -> Router {
    app.route("/", get(|| async { "MCP Server is running!" }))
        // MCP标准端点 - 初始化和主要通信（通用处理器）
        .route("/", post(handle_generic_mcp_request))
        // MCP标准端点 - SSE支持
        .route("/sse", get(handle_sse))
        // 工具端点（向后兼容）
        .route("/tools/list", post(handle_tools_list))
        .route("/tools/call", post(handle_tool_call))
}

/// 通用MCP请求处理器，能够处理任何JSON格式
async fn handle_generic_mcp_request(body: Bytes) -> axum::Json<serde_json::Value> {
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

/// SSE处理器
async fn handle_sse(headers: HeaderMap) -> Response {
    // 检查Accept头是否包含text/event-stream（宽松检查，兼容LM Studio）
    let accept_header = headers
        .get(header::ACCEPT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("*/*");

    // 只有明确拒绝时才返回错误，否则都允许
    if accept_header.contains("application/json")
        && !accept_header.contains("text/event-stream")
        && !accept_header.contains("*/*")
    {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "text/plain")
            .body("Expected text/event-stream".into())
            .unwrap();
    }

    // 首先发送初始化消息，然后保持连接
    let stream = stream::iter(vec![
        Ok::<Event, Infallible>(Event::default()
            .data("{\"jsonrpc\":\"2.0\",\"id\":null,\"result\":{\"capabilities\":{\"tools\":{\"listChanged\":false}},\"protocolVersion\":\"2025-06-18\",\"serverInfo\":{\"name\":\"local_mcp_servers\",\"version\":\"0.1.0\"}}}")
            .event("initialize"))
    ])
    .chain(
        // 然后每30秒发送一次心跳
        stream::unfold((), |_| async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Some((
                Ok::<Event, Infallible>(Event::default()
                    .data("ping")
                    .event("heartbeat")),
                (),
            ))
        })
    );

    let sse = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    );

    // 手动设置响应头
    let mut response = sse.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
        .headers_mut()
        .insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    response
        .headers_mut()
        .insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));

    response
}

/// 工具列表处理器
async fn handle_tools_list(
    json: axum::Json<
        crate::tools::tool_dto::JsonRpcRequest<crate::tools::tool_dto::ToolsListParams>,
    >,
) -> axum::Json<crate::tools::tool_dto::JsonRpcResponse<crate::tools::tool_dto::ToolsListResult>> {
    axum::Json(crate::tools::handler::handle_tools_list_internal(json.0).await)
}

/// 工具调用处理器
async fn handle_tool_call(
    json: axum::Json<
        crate::tools::tool_dto::JsonRpcRequest<crate::tools::tool_dto::ToolCallParams>,
    >,
) -> Result<
    axum::Json<crate::tools::tool_dto::JsonRpcResponse<crate::tools::tool_dto::ToolCallResult>>,
    (StatusCode, axum::Json<crate::tools::tool_dto::JsonRpcError>),
> {
    match crate::tools::handler::handle_tool_call_internal(json.0).await {
        Ok(response) => Ok(axum::Json(response)),
        Err(error) => Err((StatusCode::BAD_REQUEST, axum::Json(error))),
    }
}
