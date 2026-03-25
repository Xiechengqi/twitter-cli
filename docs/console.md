# Console Status

## Entry

Console 默认入口：

- `http://0.0.0.0:12233`

文档入口：

- `http://0.0.0.0:12233/docs`

## Navigation

当前导航栏已实现：

- `Console`
- `Commands`
- `MCP`
- `Docs`
- `Settings`
- `Logout`

## Routes

已实现页面：

- `/`
- `/login`
- `/setup/password`
- `/commands`
- `/mcp`
- `/docs`
- `/settings`

## Authentication Behavior

### Console Login

- 用户输入密码
- 验证通过后写入 cookie：
  - `twitter_cli_token=<password>`

### API and MCP

使用同一个密码作为 Bearer token：

```http
Authorization: Bearer <password>
```

### Unified Rule

只要请求携带以下任一有效凭据即通过认证：

1. `Authorization: Bearer <password>`
2. `twitter_cli_token=<password>` cookie

### Redirect Rules

未初始化密码：

- `/`
- `/commands`
- `/mcp`
- `/docs`
- `/settings`

都会跳转到：

- `/setup/password`

已初始化但未登录：

- 上述页面会跳转到 `/login`

## Console Home

当前首页已显示：

- Service Status
- Agent Browser
- Quick Actions
- Recent Executions
- VNC 状态

## Commands Page

当前已支持：

- 浏览已注册命令
- 自动生成示例参数 JSON
- 从页面直接调用 `/api/execute/:command`

## MCP Page

当前已支持：

- 展示 MCP 工具列表
- 展示 Bearer 使用方式
- 从页面直接调用 `POST /mcp`

## Settings Page

当前已支持：

- 查看并保存完整配置 JSON
- 修改密码

## Logout

当前已支持：

- 顶部导航直接登出
- 清除 `twitter_cli_token` cookie
- 跳转回 `/login`
