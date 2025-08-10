#!/bin/bash

# MCP服务器测试脚本
echo "=== MCP Server 测试脚本 ==="
echo

SERVER_URL="http://localhost:8080"

echo "1. 测试基本连接..."
curl -s -o /dev/null -w "%{http_code}" $SERVER_URL
if [ $? -eq 0 ]; then
    echo "✅ 服务器连接正常"
else
    echo "❌ 服务器连接失败"
    exit 1
fi
echo

echo "2. 测试根路径POST请求（Streamable HTTP）..."
POST_RESPONSE=$(curl -s -X POST $SERVER_URL/ -H "Content-Type: application/json")
if [[ $POST_RESPONSE == *"capabilities"* ]]; then
    echo "✅ 根路径POST请求工作正常"
    echo "   响应包含能力声明"
else
    echo "❌ 根路径POST请求失败"
    echo "   响应: $POST_RESPONSE"
fi
echo

echo "3. 测试SSE端点（MCP标准）..."
SSE_RESPONSE=$(timeout 5 curl -s -H "Accept: text/event-stream" $SERVER_URL/sse | head -n 1)
if [[ $SSE_RESPONSE == *"capabilities"* ]]; then
    echo "✅ SSE端点工作正常"
    echo "   响应: $SSE_RESPONSE"
else
    echo "❌ SSE端点响应异常"
    echo "   响应: $SSE_RESPONSE"
fi
echo

echo "4. 测试工具列表..."
TOOLS_RESPONSE=$(curl -s -X POST $SERVER_URL/tools/list \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }')

if [[ $TOOLS_RESPONSE == *"get_system_type"* && $TOOLS_RESPONSE == *"get_current_time"* ]]; then
    echo "✅ 工具列表获取成功"
    echo "   包含工具: get_system_type, get_current_time"
else
    echo "❌ 工具列表获取失败"
    echo "   响应: $TOOLS_RESPONSE"
fi
echo

echo "5. 测试系统信息工具..."
SYSTEM_RESPONSE=$(curl -s -X POST $SERVER_URL/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "get_system_type",
      "arguments": {}
    }
  }')

if [[ $SYSTEM_RESPONSE == *"os"* && $SYSTEM_RESPONSE == *"arch"* ]]; then
    echo "✅ 系统信息工具工作正常"
    echo "   响应包含OS和架构信息"
else
    echo "❌ 系统信息工具失败"
    echo "   响应: $SYSTEM_RESPONSE"
fi
echo

echo "6. 测试时间工具..."
TIME_RESPONSE=$(curl -s -X POST $SERVER_URL/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_current_time",
      "arguments": {}
    }
  }')

if [[ $TIME_RESPONSE == *"timestamp"* ]]; then
    echo "✅ 时间工具工作正常"
    echo "   响应包含时间戳信息"
else
    echo "❌ 时间工具失败"
    echo "   响应: $TIME_RESPONSE"
fi
echo

echo "7. 测试无效工具调用..."
ERROR_RESPONSE=$(curl -s -X POST $SERVER_URL/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "invalid_tool",
      "arguments": {}
    }
  }')

if [[ $ERROR_RESPONSE == *"error"* && $ERROR_RESPONSE == *"Unknown tool"* ]]; then
    echo "✅ 错误处理工作正常"
    echo "   正确返回了未知工具错误"
else
    echo "❌ 错误处理异常"
    echo "   响应: $ERROR_RESPONSE"
fi
echo

echo "=== 测试完成 ==="
echo "如果所有测试都显示 ✅，说明MCP服务器工作正常，可以与LM Studio等客户端集成。"
