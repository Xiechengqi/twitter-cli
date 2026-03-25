type Lang = 'en' | 'zh';

const dict = {
  nav: {
    en: { console: 'Console', commands: 'Commands', mcp: 'MCP', docs: 'Docs', settings: 'Settings', logout: 'Logout', login: 'Login', setup_password: 'Setup Password' },
    zh: { console: '控制台', commands: '命令', mcp: 'MCP', docs: '文档', settings: '设置', logout: '登出', login: '登录', setup_password: '设置密码' },
  },
  theme: {
    en: { light: 'Light', dark: 'Dark', auto: 'Auto' },
    zh: { light: '亮色', dark: '暗色', auto: '自动' },
  },
  home: {
    en: {
      title: 'Console', tagline: 'Local Twitter automation control plane backed by ', tagline_suffix: '.',
      service_status: 'Service Status', agent_browser: 'Agent Browser', quick_actions: 'Quick Actions',
      recent_executions: 'Recent Executions', not_set: 'not set',
      action_commands: 'Run profile, timeline, search, and write commands',
      action_mcp: 'Review MCP tools and auth model',
      action_settings: 'Adjust server, agent-browser, and auth settings',
      vnc_preview: 'Embedded preview from ', vnc_not_configured: 'VNC is not configured or embedding is disabled.',
      dt_api: 'API', dt_docs: 'Docs', dt_config: 'Config', dt_binary: 'Binary', dt_cdp_url: 'CDP URL', dt_session: 'Session',
    },
    zh: {
      title: '控制台', tagline: '基于 ', tagline_suffix: ' 的本地 Twitter 自动化控制面板。',
      service_status: '服务状态', agent_browser: 'Agent Browser', quick_actions: '快捷操作',
      recent_executions: '最近执行', not_set: '未设置',
      action_commands: '运行 profile、timeline、search 和写入命令',
      action_mcp: '查看 MCP 工具和认证模型',
      action_settings: '调整服务器、agent-browser 和认证设置',
      vnc_preview: '嵌入预览来自 ', vnc_not_configured: 'VNC 未配置或嵌入已禁用。',
      dt_api: 'API', dt_docs: '文档', dt_config: '配置', dt_binary: '可执行文件', dt_cdp_url: 'CDP URL', dt_session: '会话',
    },
  },
  login: {
    en: { title: 'Login', description: 'Use the Console password. The same credential also works as API and MCP Bearer token.', password: 'Password', submit: 'Login' },
    zh: { title: '登录', description: '使用控制台密码。同一凭据也可用作 API 和 MCP Bearer token。', password: '密码', submit: '登录' },
  },
  setup_password: {
    en: { title: 'Setup Password', description: 'First run requires a password. This password will also act as the API and MCP Bearer token.', password: 'Password', submit: 'Save Password' },
    zh: { title: '设置密码', description: '首次运行需要设置密码。此密码也将用作 API 和 MCP Bearer token。', password: '密码', submit: '保存密码' },
  },
  commands: {
    en: { title: 'Command Runner', description: 'Run any registered command through the same API used by CLI and MCP mappings.', command_label: 'Command', execute: 'Execute', running: ' Running\u2026', registered: 'Registered Commands' },
    zh: { title: '命令执行器', description: '通过 CLI 和 MCP 映射使用的同一 API 运行任何已注册命令。', command_label: '命令', execute: '执行', running: ' 执行中\u2026', registered: '已注册命令' },
  },
  mcp: {
    en: { title: 'MCP', description: 'All MCP tools use the same password as Console and API.', endpoint: 'Endpoint: ', tool_index: 'Tool index: ', tool_label: 'Tool', arguments_label: 'Arguments (JSON)', call_tool: 'Call Tool', tools_heading: 'Tools' },
    zh: { title: 'MCP', description: '所有 MCP 工具使用与控制台和 API 相同的密码。', endpoint: '端点：', tool_index: '工具索引：', tool_label: '工具', arguments_label: '参数 (JSON)', call_tool: '调用工具', tools_heading: '工具列表' },
  },
  settings: {
    en: { title: 'Settings', server: 'Server', host: 'Host', port: 'Port', agent_browser: 'Agent Browser', binary: 'Binary', cdp_url: 'CDP URL', session_name: 'Session Name', timeout: 'Timeout (seconds)', vnc: 'VNC', url: 'URL', username: 'Username', password: 'Password', embed: 'Embed', yes: 'Yes', no: 'No', save: 'Save Config', reset: 'Reset', change_password: 'Change Password', old_password: 'Old Password', new_password: 'New Password' },
    zh: { title: '设置', server: '服务器', host: '主机', port: '端口', agent_browser: 'Agent Browser', binary: '可执行文件', cdp_url: 'CDP URL', session_name: '会话名称', timeout: '超时（秒）', vnc: 'VNC', url: 'URL', username: '用户名', password: '密码', embed: '嵌入', yes: '是', no: '否', save: '保存配置', reset: '重置', change_password: '修改密码', old_password: '旧密码', new_password: '新密码' },
  },
  docs: {
    en: { title: 'Docs', description: 'Shared source of truth for commands, MCP tools, and skills.', command: 'Command', category: 'Category', mode: 'Mode', summary: 'Summary', skills: 'Skills' },
    zh: { title: '文档', description: '命令、MCP 工具和技能的统一参考。', command: '命令', category: '分类', mode: '模式', summary: '摘要', skills: '技能' },
  },
  components: {
    en: { no_executions: 'No commands have been executed yet.', when: 'When', source: 'Source', command: 'Command', status: 'Status', summary_heading: 'Summary', just_now: 'just now', status_ok: 'ok', status_err: 'error', minutes_ago: 'm ago', hours_ago: 'h ago', days_ago: 'd ago' },
    zh: { no_executions: '尚无已执行的命令。', when: '时间', source: '来源', command: '命令', status: '状态', summary_heading: '摘要', just_now: '刚刚', status_ok: '成功', status_err: '错误', minutes_ago: '分钟前', hours_ago: '小时前', days_ago: '天前' },
  },
} as const;

export type Translations = typeof dict;
export type Section = keyof Translations;

export function t(lang: Lang) {
  return {
    nav: dict.nav[lang],
    theme: dict.theme[lang],
    home: dict.home[lang],
    login: dict.login[lang],
    setup_password: dict.setup_password[lang],
    commands: dict.commands[lang],
    mcp: dict.mcp[lang],
    settings: dict.settings[lang],
    docs: dict.docs[lang],
    components: dict.components[lang],
  };
}

export type { Lang };
