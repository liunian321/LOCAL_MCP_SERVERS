# MCP Server 使用说明

## 概述

这是一个按照 Model Context Protocol (MCP) 2025-06-18 标准开发的服务器，提供了系统信息和时间查询工具，完全兼容 LM Studio 等 MCP 客户端。

## 启动服务器

```bash
cargo run
```

服务器将在配置文件 `config.yaml` 指定的端口上启动 (默认端口可在配置文件中设置)。

## 可用的 API 端点

### 1. 服务器信息端点（Streamable HTTP）

**POST** `/`

LM Studio 首先尝试的端点，返回服务器信息和能力声明。

**响应示例：**

```json
{
  "jsonrpc": "2.0",
  "result": {
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
  }
}
```

### 2. SSE 连接端点（MCP 标准）

**GET** `/sse`

支持 Server-Sent Events 的 MCP 标准端点，用于与 LM Studio 等客户端建立流式连接。当客户端回退到 SSE 时使用。

**响应示例：**

```
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive

event: initialize
data: {"jsonrpc":"2.0","id":null,"result":{"capabilities":{"tools":{"listChanged":false}},"protocolVersion":"2025-06-18","serverInfo":{"name":"local_mcp_servers","version":"0.1.0"}}}

event: heartbeat
data: ping

```

### 3. 获取工具列表

**POST** `/tools/list`

返回所有可用工具的列表，使用 JSON-RPC 2.0 格式。

**请求体：**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {
    "cursor": null
  }
}
```

**响应示例：**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "get_system_type",
        "title": "系统类型信息",
        "description": "获取当前运行系统的类型信息，包括操作系统、架构",
        "inputSchema": {
          "type": "object",
          "properties": {}
        }
      },
      {
        "name": "get_current_time",
        "title": "当前时间",
        "description": "获取当前时间",
        "inputSchema": {
          "type": "object",
          "properties": {}
        }
      }
    ]
  }
}
```

### 4. 调用工具

**POST** `/tools/call`

执行指定的工具，使用 JSON-RPC 2.0 格式。

**请求体：**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "get_system_type",
    "arguments": {}
  }
}
```

**响应示例：**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"os\": \"macos\",\n  \"arch\": \"aarch64\"\n}"
      }
    ],
    "isError": false,
    "structuredContent": {
      "os": "macos",
      "arch": "aarch64",
    }
  }
}
```

## 可用工具详情

### get_system_type

获取当前运行系统的详细信息：

- **os**: 操作系统类型 (如 "macos", "linux", "windows")
- **arch**: 系统架构 (如 "aarch64", "x86_64")

此工具不需要任何输入参数。

### get_current_time

获取当前时间信息：

- **timestamp**: 格式化的时间字符串
- **format**: 时间格式说明

此工具不需要任何输入参数。

## 使用 curl 测试

### 检查服务器信息端点：

```bash
curl -X POST http://localhost:8080/
```

### 检查 SSE 端点：

```bash
curl -H "Accept: text/event-stream" http://localhost:8080/sse
```

### 获取工具列表：

```bash
curl -X POST http://localhost:8080/tools/list \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

### 调用系统类型工具：

```bash
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "get_system_type",
      "arguments": {}
    }
  }'
```

### 调用时间工具：

```bash
curl -X POST http://localhost:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_current_time",
      "arguments": {}
    }
  }'
```

## MCP 协议兼容性

此服务器完全按照 MCP 2025-06-18 协议标准实现：

### 核心特性：

- **JSON-RPC 2.0 格式**：所有请求和响应都使用标准的 JSON-RPC 2.0 格式
- **工具发现**：支持 `tools/list` 方法获取可用工具列表
- **工具执行**：支持 `tools/call` 方法执行指定工具
- **SSE 支持**：提供 `/sse` 端点支持 Server-Sent Events，兼容 MCP 客户端
- **标准化数据结构**：使用符合 MCP 规范的工具定义和响应格式

### 传输协议支持：

- **HTTP POST**：标准的 JSON-RPC 2.0 over HTTP
- **Server-Sent Events (SSE)**：支持流式连接，符合 MCP 传输规范
- **能力声明**：通过 SSE 端点声明服务器能力

### 数据结构特性：

- **工具定义**：包含 `name`、`title`、`description`、`inputSchema` 等标准字段
- **输入验证**：使用 JSON Schema 定义工具参数
- **结构化响应**：同时提供文本内容和结构化数据
- **错误处理**：符合 JSON-RPC 2.0 的错误响应格式

### 扩展性：

- **模块化设计**：公共 DTO 结构位于 `tool_dto.rs`，各工具专注于自身功能
- **易于扩展**：可以轻松添加新的工具而不影响现有代码
- **类型安全**：使用 Rust 的类型系统确保数据结构的正确性

## 与 LM Studio 集成

此服务器现在支持与 LM Studio 等 MCP 客户端的标准集成：

### 配置方法：

1. 启动服务器：`cargo run`
2. 在 LM Studio 中配置 MCP 服务器 URL：`http://localhost:8080`
3. LM Studio 将自动检测 SSE 支持并建立连接

### 支持的传输方式：

- **Streamable HTTP**：通过 `/sse` 端点
- **标准 HTTP**：回退到常规的 JSON-RPC POST 请求

### 错误处理：

- 如果客户端不支持 SSE，服务器会正确返回 HTTP 405（当访问 SSE 端点时不包含正确的 Accept 头）
- 提供清晰的错误消息和状态码
- 支持客户端回退到标准 HTTP 传输

### 故障排除：

如果 LM Studio 显示连接错误：

1. 确保服务器正在运行：`cargo run`
2. 检查端口是否正确（默认 8080）
3. 确认防火墙没有阻止连接
4. 查看 LM Studio 日志了解具体错误信息

可以与任何支持 MCP 协议的 AI 应用程序和客户端集成使用，包括 LM Studio、Claude Desktop 等。
