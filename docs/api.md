# API Status

## Goal

同一个服务当前已经同时提供：

- Web Console
- HTTP API
- MCP
- Docs

默认监听：

- `0.0.0.0:12233`

`serve` 现在支持运行时参数覆盖：

- `--host`
- `--port`

## Public Routes

- `GET /health`
- `GET /docs`
- `GET /api/bootstrap`
- `POST /api/setup/password`
- `POST /api/login`

## Protected Routes

- `POST /api/logout`
- `GET /api/config`
- `POST /api/config`
- `GET /api/commands`
- `GET /api/history`
- `POST /api/execute/:command`
- `GET /api/mcp/tools`
- `GET /api/skills`
- `POST /api/password/change`
- `POST /mcp`

## Bootstrap API

`GET /api/bootstrap`

返回：

- 是否首次启动
- 是否要求设置密码
- 当前服务地址信息
- `agent-browser` 探测状态
- VNC 是否配置

## Password Setup

`POST /api/setup/password`

请求：

```json
{
  "password": "your-password"
}
```

行为：

- 写入 `auth.password`
- 设置 `auth.password_changed = true`

## Login

`POST /api/login`

请求：

```json
{
  "password": "your-password"
}
```

成功行为：

- 返回 `ok: true`
- 写入 cookie `twitter_cli_token=<password>`

## Logout

`POST /api/logout`

成功行为：

- 返回 `logged_out: true`
- 清除 `twitter_cli_token` cookie

## Execute API

`POST /api/execute/:command`

请求体：

```json
{
  "params": {
    "username": "openai",
    "limit": 20
  },
  "format": "json"
}
```

成功响应：

```json
{
  "ok": true,
  "data": [],
  "meta": {
    "site": "twitter",
    "command": "followings"
  }
}
```

失败响应：

```json
{
  "ok": false,
  "error": {
    "code": "AUTH_REQUIRED",
    "message": "authentication required"
  },
  "meta": {
    "site": "twitter"
  }
}
```

## Execution History

`GET /api/history`

返回最近命令执行记录，字段包括：

- `timestamp`
- `source`
- `command`
- `ok`
- `summary`

## Password Change

`POST /api/password/change`

请求：

```json
{
  "old_password": "old",
  "new_password": "new"
}
```

行为：

- 立即替换配置中的密码
- 旧 cookie 失效
- 旧 API Bearer 失效
- 旧 MCP Bearer 失效

## Docs Route

文档固定在：

- `GET /docs`

并由 Console 导航栏统一跳转。
