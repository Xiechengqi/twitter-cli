'use client';

import { useEffect, useState, useMemo } from 'react';
import { ChevronDown } from 'lucide-react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { VncEmbed } from '@/components/vnc-embed';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { CommandSpec } from '@/lib/types';

function buildExample(cmd: CommandSpec): string {
  const map: Record<string, unknown> = {};
  for (const p of cmd.params) {
    const examples: Record<string, unknown> = {
      username: 'OpenAI',
      query: 'openai',
      url: 'https://x.com/OpenAI/status/2033953592424731072',
      text: 'hello from twitter-cli',
      texts: ['hello from twitter-cli', 'follow-up post'],
      type: 'for-you',
      limit: 5,
    };
    map[p.name] = examples[p.name] ?? '';
  }
  return JSON.stringify(map, null, 2);
}

function buildCliCommand(name: string, params: Record<string, unknown>): string {
  const parts = ['twitter-cli', name];
  for (const [key, val] of Object.entries(params)) {
    if (val === '' || val === undefined || val === null) continue;
    if (Array.isArray(val)) {
      parts.push(`--${key}`, JSON.stringify(val));
    } else {
      parts.push(`--${key}`, String(val));
    }
  }
  return parts.join(' ');
}

export default function CommandsPage() {
  const { lang } = useLang();
  const tr = t(lang).commands;
  const [commands, setCommands] = useState<CommandSpec[]>([]);
  const [selected, setSelected] = useState('');
  const [params, setParams] = useState<Record<string, string>>({});
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState('');
  const [cliCmd, setCliCmd] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getCommands();
        setCommands(res.data);
        if (res.data.length > 0) setSelected(res.data[0].name);
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

  const cmd = useMemo(() => commands.find((c) => c.name === selected), [commands, selected]);

  const handleParamChange = (name: string, value: string) => {
    setParams((prev) => ({ ...prev, [name]: value }));
  };

  const parseParams = (): Record<string, unknown> | null => {
    if (!cmd) return null;
    const parsed: Record<string, unknown> = {};
    for (const p of cmd.params) {
      const val = (params[p.name] || '').trim();
      if (!val) continue;
      if (p.kind === 'integer') {
        parsed[p.name] = parseInt(val, 10);
      } else if (p.kind === 'array') {
        try { parsed[p.name] = JSON.parse(val); }
        catch (e) { setResult(`Invalid JSON for ${p.name}: ${e}`); return null; }
      } else {
        parsed[p.name] = val;
      }
    }
    return parsed;
  };

  const handleExecute = async () => {
    const parsed = parseParams();
    if (!parsed) return;

    setCliCmd(buildCliCommand(selected, parsed));
    setRunning(true);
    setResult('');
    try {
      const res = await api.executeCommand(selected, parsed);
      setResult(JSON.stringify(res, null, 2));
    } catch (e) {
      setResult(`Network error: ${e}`);
    } finally {
      setRunning(false);
    }
  };

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 flex justify-center"><Spinner /></main>
      </>
    );
  }

  return (
    <>
      <Nav authenticated />
      <main className="w-[80vw] mx-auto px-4 sm:px-6 py-16">
        <div className="grid gap-6 lg:grid-cols-[1fr_320px]">
          {/* Left: executor */}
          <Card hover={false}>
            <h1 className="text-2xl font-bold text-slate-900 dark:text-white mb-2">{tr.title}</h1>
            <p className="text-sm text-slate-500 dark:text-slate-400 mb-6">{tr.description}</p>

            <div className="space-y-4">
              <div>
                <label>{tr.command_label}</label>
                <div className="relative">
                  <select
                    value={selected}
                    onChange={(e) => { setSelected(e.target.value); setParams({}); setResult(''); setCliCmd(''); }}
                    className="mt-1 appearance-none pr-10"
                  >
                    {commands.map((c) => (
                      <option key={c.name} value={c.name}>
                        {c.name} — {c.summary}
                      </option>
                    ))}
                  </select>
                  <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 mt-0.5 h-4 w-4 text-slate-400 pointer-events-none" />
                </div>
              </div>

              {cmd?.params.map((p) => (
                <div key={p.name}>
                  <label>
                    {p.name}{p.required && <span className="text-red-500 ml-1">*</span>}
                  </label>
                  {p.kind === 'array' ? (
                    <textarea
                      rows={2}
                      placeholder={`${p.description} (JSON array)`}
                      value={params[p.name] || ''}
                      onChange={(e) => handleParamChange(p.name, e.target.value)}
                      className="mt-1"
                    />
                  ) : (
                    <input
                      type={p.kind === 'integer' ? 'number' : 'text'}
                      placeholder={p.description}
                      value={params[p.name] || ''}
                      onChange={(e) => handleParamChange(p.name, e.target.value)}
                      className="mt-1"
                    />
                  )}
                </div>
              ))}

              <button
                onClick={handleExecute}
                disabled={running}
                className="btn-primary"
              >
                {running ? <><Spinner /> {tr.running}</> : tr.execute}
              </button>

              {cliCmd && (
                <pre className="mt-4 text-xs"><span className="text-slate-400">$ </span>{cliCmd}</pre>
              )}

              {result && <pre className="mt-2 max-h-96 overflow-auto">{result}</pre>}

              <VncEmbed />
            </div>
          </Card>

          {/* Right: command list */}
          <Card hover={false}>
            <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-4">{tr.registered}</h2>
            <div className="space-y-2">
              {commands.map((c) => (
                <details key={c.name} className="group border border-slate-100 dark:border-slate-700 rounded-lg">
                  <summary className="flex items-center justify-between p-3 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-700/50 rounded-lg transition-colors">
                    <span className="font-semibold text-sm text-slate-900 dark:text-white">{c.name}</span>
                    <span className="text-xs text-slate-500 dark:text-slate-400">{c.execution_mode}</span>
                  </summary>
                  <div className="px-3 pb-3 text-sm text-slate-600 dark:text-slate-300">
                    <p className="mb-2">{c.summary}</p>
                    <pre className="text-xs">{buildExample(c)}</pre>
                  </div>
                </details>
              ))}
            </div>
          </Card>
        </div>
      </main>
    </>
  );
}
