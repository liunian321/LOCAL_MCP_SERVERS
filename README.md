# Local MCP Server

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Custom-blue.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-2025--06--18-green.svg)](https://modelcontextprotocol.io/)

这是一个兼容 Model Context Protocol (MCP) 2025-06-18 标准，基于 Rust 和 Axum 框架构建的轻量级高并发 MCP 服务器, 支持 LM Studio 等 MCP 客户端。

## 快速开始

### 前置要求
- Rust 1.70+ 
- Cargo

### 安装步骤

1. **配置服务器**
   ```bash
   cp config_example.yaml config.yaml
   # 编辑 config.yaml 文件，设置端口等参数
   ```

2. **启动服务器**
   ```bash
   cargo run
   ```

3. **配置客户端**
   - 复制 `mcp.json.example` 为 `mcp.json`
   - 调整端口号配置
   - 在 MCP 客户端中加载配置

## 配置

### 配置文件 (config.yaml)

⚠️ **注意**: 配置文件必须放在项目根目录下

```yaml
# 服务器监听端口
listen_port: 3000
```

### 客户端配置 (mcp.json)

```json
{
  "mcpServers": {
    "local_mcp_server": {
      "url": "http://127.0.0.1:3000/"
    }
  }
}
```

## 工具列表

### 系统工具
- 系统信息获取
- 时间工具
- 文件读取
- 文件列表

### 网络工具
- Ping 测试
- 当前 IP 查询和域名 IP 查询

### 其他工具
- 随机字符串生成

## 开发

### 项目结构
```
src/
├── main.rs              # 主程序入口
├── config/              # 配置管理
├── router/              # 路由处理
├── tools/               # 工具实现
│   ├── public/          # 公共工具
│   │   ├── network/     # 网络工具
│   │   └── system/      # 系统工具
│   └── handler.rs       # 工具处理器
```

### 使用说明
1. 执行命令,下载rust 和 cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. 复制 config_example.yaml 为 config.yaml

3. 在项目目录执行
```bash
cargo run
```

4. 在LM Studio中配置MCP服务器, 配置文件为mcp.json

5. 在LM Studio中点击按钮启用 mcp 服务器.

## 许可证

本项目采用自定义许可证。**任何修改、二次开发或商业使用都需要原作者明确许可**。

详细条款请查看 [LICENSE](LICENSE) 文件。

## 支持

如有问题或建议，请提交 [Issue](../../issues)。

---

**注意**: 本项目仅供学习和个人使用。商业用途或二次开发请联系原作者。