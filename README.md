# twitter-cli

`twitter-cli` 是一个独立的 Twitter 原生 CLI 与本地控制面服务。

它把 `/data/projects/opencli/src/clis/twitter` 的 Twitter 业务能力迁到独立仓库，并把原先依赖 Playwright 的浏览器层替换成 `agent-browser` CLI。

## 当前状态

当前仓库已经包含可运行实现，不再只是规划文档。

已完成能力：

- 独立 Rust CLI：`describe`、`execute`、`serve`
- 本地 HTTP 服务
- Web Console（含 History 页面）
- Docs 页面
- MCP 工具暴露（28 个工具）
- Skill catalog（3 个技能）
- 统一 manifest / describe 输出
- 共享密码认证模型
- `agent-browser` 绑定层
- 执行历史记录
- Wave 1-4 命令全部实现（28 个命令）

## 已实现命令

只读命令（Wave 1-4）：

- `profile` - 获取用户资料
- `timeline` - 获取时间线（for-you/following）
- `trending` - 获取热门话题
- `bookmarks` - 获取书签列表
- `search` - 搜索推文
- `followers` - 获取粉丝列表
- `followings` - 获取关注列表
- `likes` - 获取点赞列表
- `notifications` - 获取通知
- `article` - 读取推文长文章
- `download` - 提取媒体 URL
- `tweet` - 获取单条推文详情
- `replies` - 获取推文回复

写命令（Wave 2-4）：

- `like` / `unlike` - 点赞/取消点赞
- `bookmark` / `unbookmark` - 书签/取消书签
- `follow` / `unfollow` - 关注/取消关注
- `retweet` - 转发
- `post` - 发推文
- `reply` - 回复推文
- `thread` - 发推文串
- `delete` - 删除推文
- `block` / `unblock` - 拉黑/取消拉黑
- `hide_reply` - 隐藏回复
- `accept_dm` - 批量接受 DM 请求（支持多关键词 OR 匹配）
- `reply_dm` - 批量发送 DM

## 已实现 MCP Tools

只读工具：
- `twitter_profile`
- `twitter_timeline`
- `twitter_trending`
- `twitter_search`
- `twitter_followers`
- `twitter_followings`
- `twitter_bookmarks`
- `twitter_likes`
- `twitter_notifications`
- `twitter_article`
- `twitter_download`
- `twitter_tweet`
- `twitter_replies`

写操作工具：
- `twitter_like` / `twitter_unlike`
- `twitter_bookmark` / `twitter_unbookmark`
- `twitter_follow` / `twitter_unfollow`
- `twitter_retweet`
- `twitter_post`
- `twitter_reply`
- `twitter_thread`
- `twitter_delete`
- `twitter_block` / `twitter_unblock`
- `twitter_hide_reply`
- `twitter_accept_dm`
- `twitter_reply_dm`

## 已实现 Skills

- `research_account`
- `monitor_keyword`
- `prepare_reply_context`

## 本地运行

构建：

```bash
cargo build --offline
```

查看自描述：

```bash
target/debug/twitter-cli describe --json
```

执行命令：

```bash
target/debug/twitter-cli execute profile --params '{"username":"OpenAI"}'
```

启动服务：

```bash
target/debug/twitter-cli serve
```

覆盖监听地址：

```bash
target/debug/twitter-cli serve --host 0.0.0.0 --port 12233
```

默认地址：

- Console: `http://0.0.0.0:12233`
- Docs: `http://0.0.0.0:12233/docs`

## 配置

配置文件路径：

- `${HOME}/.config/twitter-cli/config.toml`

首次启动会自动：

1. 创建配置目录
2. 探测 `agent-browser`
3. 写入默认配置
4. 要求先设置 Console 密码

认证模型：

- Console Cookie：`twitter_cli_token`
- API Bearer：`Authorization: Bearer <password>`
- MCP Bearer：`Authorization: Bearer <password>`

## HTTP / MCP 概览

公共接口：

- `GET /health`
- `GET /docs`
- `GET /api/bootstrap`
- `POST /api/setup/password`
- `POST /api/login`

受保护接口：

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

MCP 已实现方法：

- `initialize`
- `ping`
- `notifications/initialized`
- `tools/list`
- `tools/call`

## 测试

当前已落地的本地验证方式：

```bash
cargo test --offline
cargo build --offline
```

测试覆盖：

- manifest 注册
- MCP schema 生成
- MCP 错误响应结构
- Commands 页示例 payload
- 执行结果摘要逻辑

## 规划文档

- [docs/architecture.md](/data/projects/twitter-cli/docs/architecture.md)
- [docs/config.md](/data/projects/twitter-cli/docs/config.md)
- [docs/console.md](/data/projects/twitter-cli/docs/console.md)
- [docs/api.md](/data/projects/twitter-cli/docs/api.md)
- [docs/mcp-skill.md](/data/projects/twitter-cli/docs/mcp-skill.md)
- [docs/testing.md](/data/projects/twitter-cli/docs/testing.md)
