'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { ArrowRight, Terminal, Wrench, Settings } from 'lucide-react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { StatusDot } from '@/components/status-dot';
import { ExecutionTable } from '@/components/execution-table';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AppConfig, ExecutionRecord } from '@/lib/types';

function extractCdpHost(url: string): string {
  try {
    const u = new URL(url.replace(/^ws:/, 'http:').replace(/^wss:/, 'https:'));
    return `${u.hostname}:${u.port}`;
  } catch {
    return url;
  }
}

export default function HomePage() {
  const { lang } = useLang();
  const tr = t(lang).home;
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [records, setRecords] = useState<ExecutionRecord[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const bs = await api.bootstrap();
        if (bs.password_required) {
          window.location.href = '/setup/password';
          return;
        }
        const [cfgRes, histRes] = await Promise.all([api.getConfig(), api.getHistory()]);
        setConfig(cfgRes.data);
        setRecords(histRes.data);
      } catch {
        // 401 handled by api wrapper
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 flex justify-center">
          <Spinner />
        </main>
      </>
    );
  }

  const baseUrl = config ? `http://${config.server.host}:${config.server.port}` : '';

  return (
    <>
      <Nav authenticated />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16">
        <div className="mb-12 text-center">
          <h1 className="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight gradient-text mb-4">
            twitter-cli
          </h1>
          <p className="text-lg text-slate-500 max-w-2xl mx-auto">
            {tr.tagline}<code className="font-semibold text-brand-600">agent-browser</code>{tr.tagline_suffix}
          </p>
        </div>

        <div className="grid gap-6 md:grid-cols-2">
          {/* Service Status */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.service_status}</h2>
            <dl className="space-y-3">
              <div><dt>{tr.dt_api}</dt><dd><code>{baseUrl}</code></dd></div>
              <div><dt>{tr.dt_docs}</dt><dd><code>{baseUrl}/docs</code></dd></div>
              <div><dt>{tr.dt_config}</dt><dd><code>~/.config/twitter-cli/config.toml</code></dd></div>
            </dl>
          </Card>

          {/* Agent Browser */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.agent_browser}</h2>
            <dl className="space-y-3">
              <div>
                <dt>{tr.dt_binary}</dt>
                <dd>
                  {config?.agent_browser.binary
                    ? <code>{config.agent_browser.binary}</code>
                    : <span className="text-amber-600 text-xs">{tr.binary_not_set} <a href="https://github.com/vercel-labs/agent-browser" target="_blank" rel="noopener noreferrer" className="underline hover:text-amber-700">github.com/vercel-labs/agent-browser</a></span>
                  }
                </dd>
              </div>
              <div>
                <dt>{tr.dt_cdp_url}</dt>
                <dd>
                  <StatusDot ok={!!config?.agent_browser.cdp_url} />
                  {config?.agent_browser.cdp_url
                    ? <code>{extractCdpHost(config.agent_browser.cdp_url)}</code>
                    : <span className="text-amber-600 text-xs">{tr.cdp_not_set} <code>agent-browser connect &lt;port|url&gt;</code></span>
                  }
                </dd>
              </div>
            </dl>
          </Card>

          {/* Quick Actions */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.quick_actions}</h2>
            <div className="space-y-3">
              {[
                { href: '/commands', icon: Terminal, label: tr.action_commands },
                { href: '/mcp', icon: Wrench, label: tr.action_mcp },
                { href: '/settings', icon: Settings, label: tr.action_settings },
              ].map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className="group flex items-center gap-3 p-3 -mx-3 rounded-lg hover:bg-slate-50 transition-colors"
                >
                  <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-brand-50 text-brand-600">
                    <item.icon className="h-5 w-5" />
                  </div>
                  <span className="text-sm font-medium text-slate-700 flex-1">{item.label}</span>
                  <ArrowRight className="h-4 w-4 text-slate-400 transition-transform group-hover:translate-x-1" />
                </Link>
              ))}
            </div>
          </Card>

          {/* Recent Executions */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.recent_executions}</h2>
            <ExecutionTable records={records} />
          </Card>
        </div>
      </main>
    </>
  );
}
