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
import type { AppConfig, ExecutionRecord, BootstrapInfo } from '@/lib/types';

export default function HomePage() {
  const { lang } = useLang();
  const tr = t(lang).home;
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [records, setRecords] = useState<ExecutionRecord[]>([]);
  const [bootstrap, setBootstrap] = useState<BootstrapInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const bs = await api.bootstrap();
        setBootstrap(bs);
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
  const vncConfigured = config ? !!(config.vnc.url && config.vnc.embed) : false;

  return (
    <>
      <Nav authenticated />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16">
        <div className="mb-12 text-center">
          <h1 className="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight gradient-text mb-4">
            twitter-cli
          </h1>
          <p className="text-lg text-slate-500 dark:text-slate-400 max-w-2xl mx-auto">
            {tr.tagline}<code className="font-semibold text-brand-600 dark:text-brand-400">agent-browser</code>{tr.tagline_suffix}
          </p>
        </div>

        <div className="grid gap-6 md:grid-cols-2">
          {/* Service Status */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-4">{tr.service_status}</h2>
            <dl className="space-y-3">
              <div><dt>{tr.dt_api}</dt><dd><code>{baseUrl}</code></dd></div>
              <div><dt>{tr.dt_docs}</dt><dd><code>{baseUrl}/docs</code></dd></div>
              <div><dt>{tr.dt_config}</dt><dd><code>~/.config/twitter-cli/config.toml</code></dd></div>
            </dl>
          </Card>

          {/* Agent Browser */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-4">{tr.agent_browser}</h2>
            <dl className="space-y-3">
              <div><dt>{tr.dt_binary}</dt><dd><code>{config?.agent_browser.binary}</code></dd></div>
              <div>
                <dt>{tr.dt_cdp_url}</dt>
                <dd>
                  <StatusDot ok={!!config?.agent_browser.cdp_url} />
                  <code>{config?.agent_browser.cdp_url || tr.not_set}</code>
                </dd>
              </div>
              <div><dt>{tr.dt_session}</dt><dd><code>{config?.agent_browser.session_name}</code></dd></div>
            </dl>
          </Card>

          {/* Quick Actions */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-4">{tr.quick_actions}</h2>
            <div className="space-y-3">
              {[
                { href: '/commands', icon: Terminal, label: tr.action_commands },
                { href: '/mcp', icon: Wrench, label: tr.action_mcp },
                { href: '/settings', icon: Settings, label: tr.action_settings },
              ].map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className="group flex items-center gap-3 p-3 -mx-3 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50 transition-colors"
                >
                  <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-brand-50 dark:bg-brand-950 text-brand-600 dark:text-brand-400">
                    <item.icon className="h-5 w-5" />
                  </div>
                  <span className="text-sm font-medium text-slate-700 dark:text-slate-200 flex-1">{item.label}</span>
                  <ArrowRight className="h-4 w-4 text-slate-400 transition-transform group-hover:translate-x-1" />
                </Link>
              ))}
            </div>
          </Card>

          {/* Recent Executions */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-4">{tr.recent_executions}</h2>
            <ExecutionTable records={records} />
          </Card>

          {/* VNC */}
          <Card className="md:col-span-2">
            <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-4 flex items-center gap-2">
              <StatusDot ok={vncConfigured} />
              VNC
            </h2>
            {vncConfigured && config ? (
              <>
                <p className="text-sm text-slate-500 dark:text-slate-400 mb-3">
                  {tr.vnc_preview}{config.vnc.url}
                </p>
                <iframe
                  src={config.vnc.url}
                  className="w-full h-80 border border-slate-200 dark:border-slate-700 rounded-xl bg-white dark:bg-slate-900"
                />
              </>
            ) : (
              <p className="text-sm text-slate-500 dark:text-slate-400">{tr.vnc_not_configured}</p>
            )}
          </Card>
        </div>
      </main>
    </>
  );
}
