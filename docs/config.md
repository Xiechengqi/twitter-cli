# Config Plan

## Config Path

配置文件固定为：

- `${HOME}/.config/twitter-cli/config.toml`

相关目录建议：

```text
${HOME}/.config/twitter-cli/
  config.toml
```

## Config Model

推荐配置结构：

```toml
[server]
host = "127.0.0.1"
port = 12233

[auth]
password = ""
password_changed = false

[agent_browser]
binary = "/usr/local/bin/agent-browser"
cdp_url = ""
session_name = "twitter-cli"

[vnc]
url = ""
username = ""
password = ""
embed = true
```

## Field Rules

### `[server]`

- `host`
  默认 `127.0.0.1`
- `port`
  默认 `12233`

### `[auth]`

- `password`
  明文保存
- `password_changed`
  首次打开 Console 前必须为 `false`

说明：

- 当前项目明确以个人使用和易用性优先
- Console 密码同时作为 API 和 MCP Bearer token
- 不引入 JWT，不引入单独 token 颁发机制

### `[agent_browser]`

- `binary`
  默认启动时通过 `which agent-browser` 自动探测
- `cdp_url`
  可为空；为空时使用默认 `agent-browser` 行为
- `session_name`
  默认 `twitter-cli`

### `[vnc]`

- `url`
  VNC Web 页地址
- `username`
  VNC 登录用户名
- `password`
  VNC 登录密码
- `embed`
  是否在 Console 中以 iframe 嵌入

## First-Run Rules

### If config file does not exist

服务启动时：

1. 创建 `${HOME}/.config/twitter-cli/`
2. 执行 `which agent-browser`
3. 写入默认配置
4. `auth.password = ""`
5. `auth.password_changed = false`

### If password not initialized

满足以下任一条件：

- `auth.password == ""`
- `auth.password_changed == false`

则：

- 所有 Console 访问重定向到 `/setup/password`
- 用户必须先设置密码

## Persistence Rules

- 配置更新后原子写回 `config.toml`
- 文件权限建议为 `0600`
- 修改密码后旧密码立即失效
- 修改密码后旧 cookie / API Bearer / MCP Bearer 全部立即失效
