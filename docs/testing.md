# Testing Status

## Automated Checks

当前建议的基础检查：

```bash
cargo test --offline
cargo build --offline
```

## Covered By Unit Tests

当前单元测试已覆盖：

- manifest 中 `thread` / `delete` 注册
- MCP tool 注册
- `prepare_reply_context` skill 注册
- MCP input schema 生成
- MCP error 响应结构
- Commands 页面示例 payload 生成
- 执行结果摘要逻辑

## Covered By Manual Runtime Verification

当前已经做过的人工验证包括：

- `describe --json`
- 只读命令真实执行
- Console 登录 / 跳转 / 配置保存
- MCP `initialize`
- MCP `tools/list`
- MCP `tools/call`
- `/api/history`

## Current Limitations

以下写命令目前主要完成了实现接线与参数校验，但没有在本仓库里做自动化副作用回归：

- `like`
- `unlike`
- `bookmark`
- `unbookmark`
- `follow`
- `unfollow`
- `post`
- `reply`
- `thread`
- `delete`

原因：

- 它们依赖真实登录态
- 会对 Twitter 账号产生副作用
- 当前仓库没有隔离测试账号和录制用例

## Recommended Next Testing Work

如果继续补测试，优先顺序建议：

1. 为 `thread` / `delete` 加独立的无副作用 DOM 辅助函数测试
2. 增加服务层 HTTP 集成测试
3. 为 MCP `initialize` / `tools/list` / `tools/call` 增加端到端本地测试
4. 在隔离账号下做真实写命令回归
