# MCP Server 使用说明

## 概述

这是一个按照 Model Context Protocol (MCP) 2025-06-18 标准开发的 MCP 服务器，完全兼容 LM Studio 等 MCP 客户端。

## 配置文件(config.yaml)
### 注意: 配置文件需要放在当前目录下,如果缺少配置文件,服务器将无法启动
- 复制 config_example.yaml 文件并重命名为 config.yaml
- 示例配置文件:

```yaml
listen_port: 8080

```
## 工具列表
- 获取系统类型信息
- 获取当前时间

## 启动服务器

```bash
cargo run
```

## mcp.js 示例

```json
{
  "mcpServers": {
    "my_server": {
      "url": "http://127.0.0.1:8080/"
    }
  }
}

```