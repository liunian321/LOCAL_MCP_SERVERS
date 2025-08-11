# Local MCP Servers

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Custom-blue.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-2025--06--18-green.svg)](https://modelcontextprotocol.io/)

一个兼容 Model Context Protocol (MCP) 2025-06-18 标准的 Rust 服务器，支持 LM Studio 等 MCP 客户端。

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
    "local_mcp_servers": {
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

### 常用命令
```bash
cargo check      # 检查代码
cargo test       # 运行测试
cargo build      # 构建项目
cargo fmt        # 代码格式化
```

## 许可证

本项目采用自定义许可证。**任何修改、二次开发或商业使用都需要原作者明确许可**。

详细条款请查看 [LICENSE](LICENSE) 文件。

## 支持

如有问题或建议，请提交 [Issue](../../issues)。

---

**注意**: 本项目仅供学习和个人使用。商业用途或二次开发请联系原作者。