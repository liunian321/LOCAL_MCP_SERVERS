use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response, Sse, sse::Event},
};
use futures::stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;

/// SSE处理器
pub async fn handle_sse(headers: HeaderMap) -> Response {
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
