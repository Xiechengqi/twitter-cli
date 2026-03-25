# MCP And Skill Status

## Goal

`twitter-cli` 当前已经提供：

- MCP tools
- Skill catalog

两者都复用同一个服务端口和同一个认证密码。

## Authentication

MCP 与 API 共用同一个 Bearer token：

```http
Authorization: Bearer <console-password>
```

## Implemented MCP Route

- `POST /mcp`
- `GET /api/mcp/tools`

## Implemented MCP Methods

当前已实现：

- `initialize`
- `ping`
- `notifications/initialized`
- `tools/list`
- `tools/call`

其中：

- `initialize` / `ping` / `notifications/initialized` 用于最小握手
- `tools/list` 返回 tools 清单与输入 schema
- `tools/call` 复用内部命令执行器

## Implemented MCP Tools

### Read

- `twitter_profile`
- `twitter_timeline`
- `twitter_trending`
- `twitter_search`
- `twitter_followers`
- `twitter_followings`
- `twitter_bookmarks`

### Write

- `twitter_post`
- `twitter_reply`
- `twitter_thread`
- `twitter_delete`
- `twitter_follow`
- `twitter_unfollow`
- `twitter_like`
- `twitter_unlike`
- `twitter_bookmark`
- `twitter_unbookmark`

## MCP Result Shape

`tools/call` 成功结果当前包含：

- `tool`
- `command`
- `ok`
- `data`
- `structuredContent`
- `content`

失败结果当前返回 JSON-RPC `error`：

- `code`
- `message`
- `data`

## Implemented Skills

### `research_account`

组合：

- `profile`
- `timeline`
- `followings`

### `monitor_keyword`

组合：

- `search`
- `trending`

### `prepare_reply_context`

组合：

- `profile`
- `search`

## Docs Requirements

当前 `/docs` 已展示：

- command 列表
- skill 列表

当前 `/mcp` 页面已展示：

- tool 列表
- Bearer 使用说明
- 简易 MCP playground
