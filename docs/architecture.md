# Architecture Plan

## Goal

把 `twitter-cli` 规划成一个独立的站点原生子程序：

- 独立仓库
- 独立 CLI
- 独立 HTTP 服务
- 内部直接绑定 `agent-browser` CLI
- 对外暴露 API、MCP、Skill、Console

## Product Shape

`twitter-cli` 不是 workflow 文件集合，也不是宿主内置模块，而是：

- 一个独立可执行程序
- 一个本地控制面服务
- 一个 Twitter 自动化能力提供者

宿主平台与它的关系：

- 宿主负责发现、配置、加载外部站点 CLI
- `twitter-cli` 负责 Twitter 命令实现与服务暴露

## Runtime Principle

明确约束：

- 不做 Playwright 抽象兼容层
- 不做多浏览器驱动适配层
- 不做 Rust 动态插件
- 只绑定 `agent-browser` CLI

## Internal Layers

推荐保留 5 个薄层：

1. CLI layer
   负责 `describe`、`execute`、`serve`

2. HTTP server layer
   负责 Console、Docs、API、MCP、Skill 路由

3. Command layer
   负责 `timeline`、`search`、`profile`、`followings` 等命令分发

4. Twitter workflow/helper layer
   负责 Twitter 站点登录态检查、DOM 抽取、请求拼装、结果解析

5. agent-browser binding layer
   负责统一调用 `agent-browser` CLI

## Repository Layout

建议后续实现目录：

```text
twitter-cli/
  site.toml
  Cargo.toml
  README.md
  docs/
    architecture.md
    config.md
    console.md
    api.md
    mcp-skill.md
  src/
    main.rs
    cli.rs
    errors.rs
    manifest.rs
    config.rs
    auth.rs
    server/
      mod.rs
      routes.rs
      console.rs
      docs.rs
      mcp.rs
      skills.rs
    commands/
      mod.rs
      timeline.rs
      trending.rs
      search.rs
      profile.rs
      followers.rs
      followings.rs
      bookmarks.rs
      post.rs
      reply.rs
      follow.rs
      unfollow.rs
      like.rs
      unlike.rs
      bookmark.rs
      unbookmark.rs
    twitter/
      mod.rs
      auth.rs
      graphql.rs
      selectors.rs
      extract.rs
      ui.rs
    agent_browser/
      mod.rs
      client.rs
```

## Command Waves

### Wave 1

- `timeline`
- `trending`
- `search`
- `profile`
- `followers`
- `followings`

### Wave 2

- `bookmarks`
- `notifications`
- `follow`
- `unfollow`
- `like`
- `unlike`
- `bookmark`
- `unbookmark`

### Wave 3

- `post`
- `reply`
- `thread`
- `delete`

## Execution Strategy

Twitter 命令按两种模式实现：

### API-first

适合：

- `timeline`
- `search`
- `profile`
- `followers`
- `followings`
- `trending`

流程：

- 用 `agent-browser` 建立页面和 cookie context
- 读取 cookie / csrf
- 在页面上下文发请求
- 由 Rust 做结果解析和归一化

### UI-first

适合：

- `post`
- `reply`
- `follow`
- `like`
- `bookmark`

流程：

- 直接导航到目标页面
- 用 `agent-browser` 做点击、输入、等待、确认

## Why Not Dynamic Plugin

不规划 Rust 动态插件，原因：

- Rust ABI 不稳定
- 宿主与插件版本耦合重
- 崩溃会影响宿主
- 安全边界差

当前目标是独立子程序模式：

- 宿主通过进程调用或 HTTP 调用 `twitter-cli`
- `twitter-cli` 作为外部原生站点模块独立运行
