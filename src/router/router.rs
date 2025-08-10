use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};

pub fn init_bind_router(app: Router) -> Router {
    app.route("/", get(|| async { "MCP Server is running!" }))
        // MCP标准端点 - 初始化和主要通信（通用处理器）
        .route("/", post(crate::tools::mcp_handler::handle_generic_mcp_request))
        // MCP标准端点 - SSE支持
        .route("/sse", get(crate::tools::sse_handler::handle_sse))
        // 工具端点（向后兼容）
        .route("/tools/list", post(handle_tools_list))
        .route("/tools/call", post(handle_tool_call))
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
