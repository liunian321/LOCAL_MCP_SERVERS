use crate::tools::tool_dto::{ToolCallResult, ToolContent};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;
use tokio::time::{Duration, timeout};
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{
    NameServerConfig, NameServerConfigGroup, Protocol, ResolverConfig, ResolverOpts,
};

#[derive(Debug, Deserialize)]
struct ReadIpArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

#[derive(Debug, Serialize, Clone)]
struct IpLatency {
    ip: String,
    version: String,
    latency_ms: Option<u128>,
    reachable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReadIpStatus {
    Success,
    InvalidArguments,
    ResolveFailed,
    NetworkError,
}

#[derive(Debug, Serialize)]
struct ReadIpResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dns_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    records: Option<Vec<IpLatency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_ips: Option<Vec<IpLatency>>,
    status: ReadIpStatus,
}

fn error_result(message: &str, status: ReadIpStatus) -> ToolCallResult {
    ToolCallResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: Some(message.to_string()),
            data: None,
            mime_type: None,
            annotations: None,
        }],
        is_error: Some(true),
        structured_content: Some(serde_json::json!({
            "error": message,
            "status": status,
        })),
    }
}

async fn fetch_public_ip() -> Option<String> {
    // 尝试多个公共服务，任一成功即可
    let clients = vec![
        ("https://api.ipify.org?format=json", "ip"),
        ("https://ifconfig.co/json", "ip"),
    ];

    for (url, key) in clients {
        if let Ok(resp) = timeout(Duration::from_secs(5), reqwest::get(url)).await {
            if let Ok(resp) = resp {
                if resp.status().is_success() {
                    if let Ok(val) = resp.json::<serde_json::Value>().await {
                        if let Some(ip) = val.get(key).and_then(|v| v.as_str()) {
                            return Some(ip.to_string());
                        }
                    }
                }
            }
        }
    }

    // 纯文本回退
    if let Ok(resp) = timeout(
        Duration::from_secs(5),
        reqwest::get("https://ipv4.icanhazip.com"),
    )
    .await
    {
        if let Ok(resp) = resp {
            if let Ok(text) = resp.text().await {
                let t = text.trim();
                if !t.is_empty() {
                    return Some(t.to_string());
                }
            }
        }
    }

    None
}

fn parse_dns(dns: &str) -> Option<SocketAddr> {
    // 支持 "8.8.8.8" 或 "8.8.8.8:53"
    if let Ok(sa) = dns.parse::<SocketAddr>() {
        return Some(sa);
    }
    if let Ok(ip) = dns.parse::<IpAddr>() {
        return Some(SocketAddr::new(ip, 53));
    }
    None
}

async fn build_resolver(dns: Option<&str>) -> Result<(TokioAsyncResolver, Option<String>), String> {
    if let Some(d) = dns {
        let sa = parse_dns(d).ok_or_else(|| "无效的DNS地址".to_string())?;
        let mut group = NameServerConfigGroup::new();
        group.push(NameServerConfig::new(sa, Protocol::Udp));
        let cfg = ResolverConfig::from_parts(None, vec![], group);
        let resolver = TokioAsyncResolver::tokio(cfg, ResolverOpts::default());
        Ok((resolver, Some(sa.to_string())))
    } else {
        // 使用通用公共DNS作为默认（避免因系统配置API差异导致编译失败）
        let mut group = NameServerConfigGroup::new();
        let cf = SocketAddr::new("1.1.1.1".parse().unwrap(), 53);
        let gg = SocketAddr::new("8.8.8.8".parse().unwrap(), 53);
        group.push(NameServerConfig::new(cf, Protocol::Udp));
        group.push(NameServerConfig::new(gg, Protocol::Udp));
        let cfg = ResolverConfig::from_parts(None, vec![], group);
        let resolver = TokioAsyncResolver::tokio(cfg, ResolverOpts::default());
        Ok((resolver, Some("1.1.1.1,8.8.8.8".to_string())))
    }
}

async fn measure_latency(ip: IpAddr, port: u16) -> IpLatency {
    let addr = SocketAddr::new(ip, port);
    let start = Instant::now();
    let connect_fut = tokio::net::TcpStream::connect(addr);
    match timeout(Duration::from_secs(2), connect_fut).await {
        Ok(Ok(_stream)) => IpLatency {
            ip: ip.to_string(),
            version: if ip.is_ipv4() {
                "v4".into()
            } else {
                "v6".into()
            },
            latency_ms: Some(start.elapsed().as_millis()),
            reachable: true,
        },
        _ => IpLatency {
            ip: ip.to_string(),
            version: if ip.is_ipv4() {
                "v4".into()
            } else {
                "v6".into()
            },
            latency_ms: None,
            reachable: false,
        },
    }
}

async fn resolve_and_rank(
    domain: &str,
    dns: Option<&str>,
    port: u16,
) -> Result<(Vec<IpLatency>, Option<String>), String> {
    let (resolver, dns_used) = build_resolver(dns).await?;
    let lookup = resolver
        .lookup_ip(domain)
        .await
        .map_err(|e| format!("解析域名失败: {}", e))?;
    let ips: Vec<IpAddr> = lookup.iter().collect();
    if ips.is_empty() {
        return Err("未获取到任何IP地址".to_string());
    }

    // 并发测量延迟
    let mut tasks = futures::stream::FuturesUnordered::new();
    for ip in ips {
        tasks.push(measure_latency(ip, port));
    }

    let mut results: Vec<IpLatency> = Vec::new();
    while let Some(res) = tasks.next().await {
        results.push(res);
    }

    Ok((results, dns_used))
}

use futures::StreamExt;

/**
 * 处理 read ip 工具调用
 */
pub async fn handle_read_ip_tool(args: Option<serde_json::Value>) -> ToolCallResult {
    let parsed: ReadIpArgs = match args
        .map(|v| serde_json::from_value::<ReadIpArgs>(v))
        .transpose()
    {
        Ok(v) => v.unwrap_or(ReadIpArgs {
            domain: None,
            dns: None,
            port: None,
        }),
        Err(_) => {
            return error_result("参数格式错误", ReadIpStatus::InvalidArguments);
        }
    };

    if parsed.domain.is_none() {
        // 查询公网IP
        match fetch_public_ip().await {
            Some(ip) => {
                let text = format!("公网IP: {}", ip);
                let res = ReadIpResult {
                    domain: None,
                    dns_used: None,
                    public_ip: Some(ip.clone()),
                    records: None,
                    top_ips: None,
                    status: ReadIpStatus::Success,
                };
                return ToolCallResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: Some(text),
                        data: None,
                        mime_type: None,
                        annotations: None,
                    }],
                    is_error: Some(false),
                    structured_content: Some(serde_json::to_value(res).unwrap()),
                };
            }
            None => {
                return error_result("获取公网IP失败", ReadIpStatus::NetworkError);
            }
        }
    }

    // 域名查询路径
    let domain = parsed.domain.unwrap();
    let port = parsed.port.unwrap_or(80);
    match resolve_and_rank(&domain, parsed.dns.as_deref(), port).await {
        Ok((records, dns_used)) => {
            let mut reachable: Vec<&IpLatency> = records
                .iter()
                .filter(|r| r.reachable && r.latency_ms.is_some())
                .collect();
            reachable.sort_by_key(|r| r.latency_ms.unwrap());
            let top_ips: Vec<IpLatency> =
                reachable.into_iter().take(3).map(|r| r.clone()).collect();

            let res = ReadIpResult {
                domain: Some(domain.clone()),
                dns_used,
                public_ip: None,
                records: Some(records.clone()),
                top_ips: Some(top_ips.clone()),
                status: ReadIpStatus::Success,
            };

            let text = if top_ips.is_empty() {
                format!("域名 {} 解析到的IP均不可达或超时", domain)
            } else {
                let list = top_ips
                    .iter()
                    .map(|r| format!("{}({}): {}ms", r.ip, r.version, r.latency_ms.unwrap()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("域名 {} 延迟最低的三个IP: {}", domain, list)
            };

            ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(text),
                    data: None,
                    mime_type: None,
                    annotations: None,
                }],
                is_error: Some(false),
                structured_content: Some(serde_json::to_value(res).unwrap()),
            }
        }
        Err(e) => error_result(&e, ReadIpStatus::ResolveFailed),
    }
}
