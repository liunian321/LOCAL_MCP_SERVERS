use crate::tools::tool_dto::{ToolCallResult, ToolContent};
use serde::Deserialize;
use std::net::TcpStream;
use std::time::{Duration, Instant};
use url::Url;

#[derive(Debug, Deserialize)]
struct PingArgs {
    target: String,
}

/// 解析目标地址，区分网址和 IP:端口
fn parse_target(target: &str) -> Result<String, String> {
    // 检查空字符串
    if target.trim().is_empty() {
        return Err("目标地址不能为空".to_string());
    }

    // 尝试解析为 URL
    if let Ok(url) = Url::parse(target) {
        if let Some(host) = url.host_str() {
            let port = url.port().unwrap_or_else(|| match url.scheme() {
                "https" => 443,
                "http" => 80,
                _ => 80,
            });
            return Ok(format!("{}:{}", host, port));
        }
    }

    // 检查是否为 IP:端口格式
    if target.contains(':') {
        let parts: Vec<&str> = target.split(':').collect();
        if parts.len() == 2 {
            let _host = parts[0];
            if let Ok(_port) = parts[1].parse::<u16>() {
                return Ok(target.to_string());
            }
        }
    }

    // 检查是否为纯 IP 地址（默认端口 80）
    if target.parse::<std::net::IpAddr>().is_ok() {
        return Ok(format!("{}:80", target));
    }

    // 检查是否为域名（默认端口 80）
    if !target.contains(':') && target.contains('.') {
        return Ok(format!("{}:80", target));
    }

    Err("无效的目标地址格式".to_string())
}

fn ping_tool(target: &str) -> ToolCallResult {
    // 解析目标地址
    let parsed_target = match parse_target(target) {
        Ok(addr) => addr,
        Err(err_msg) => {
            let error_msg = err_msg.clone();
            return ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(err_msg),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(true),
                structured_content: Some(serde_json::json!({
                    "error": error_msg,
                    "target": target,
                    "status": "invalid_format"
                })),
            };
        }
    };

    let start_time = Instant::now();
    let timeout = Duration::from_secs(5);

    let connection_result = std::panic::catch_unwind(|| {
        // 这里系统会自动处理域名解析
        let addr_iter = std::net::ToSocketAddrs::to_socket_addrs(&parsed_target);
        match addr_iter {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    TcpStream::connect_timeout(&addr, timeout)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "无法解析地址",
                    ))
                }
            }
            Err(e) => Err(e),
        }
    });

    let duration = start_time.elapsed();

    match connection_result {
        Ok(Ok(_stream)) => {
            // 连接成功
            let result_text = format!("连接成功 - 目标: {} 延迟: {:?}", parsed_target, duration);

            let structured_result = serde_json::json!({
                "target": parsed_target,
                "latency_ms": duration.as_millis(),
                "status": "success",
                "connected": true,
                "timeout": false
            });

            ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(result_text),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(false),
                structured_content: Some(structured_result),
            }
        }
        Ok(Err(_)) => {
            // 连接失败
            let result_text = format!(
                "连接失败 - 目标: {} 尝试时间: {:?}",
                parsed_target, duration
            );

            let structured_result = serde_json::json!({
                "target": parsed_target,
                "latency_ms": duration.as_millis(),
                "status": "connection_failed",
                "connected": false,
                "timeout": false
            });

            ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(result_text),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(true),
                structured_content: Some(structured_result),
            }
        }
        Err(_) => {
            // 超时或其他错误
            let result_text = format!("连接超时 - 目标: {} 超时时间: {:?}", parsed_target, timeout);

            let structured_result = serde_json::json!({
                "target": parsed_target,
                "latency_ms": timeout.as_millis(),
                "status": "timeout",
                "connected": false,
                "timeout": true
            });

            ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(result_text),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(true),
                structured_content: Some(structured_result),
            }
        }
    }
}
/**
 * 处理 ping 工具调用
 */
pub fn handle_ping_tool(_args: Option<serde_json::Value>) -> ToolCallResult {
    let args: PingArgs = match _args {
        Some(args) => match serde_json::from_value(args) {
            Ok(parsed_args) => parsed_args,
            Err(_) => {
                return ToolCallResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: Some("参数格式错误，需要包含 target 字段".to_string()),
                        data: None,
                        mime_type: None,
                        annotations: None,
                    }],
                    is_error: Some(true),
                    structured_content: Some(serde_json::json!({
                        "error": "参数格式错误",
                        "status": "invalid_format"
                    })),
                };
            }
        },
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
                structured_content: Some(serde_json::json!({
                    "error": "缺少参数",
                    "status": "missing_arguments"
                })),
            };
        }
    };

    ping_tool(&args.target)
}
