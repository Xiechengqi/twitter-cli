'use client';

import { useEffect, useState, useCallback, useMemo } from 'react';
import { ChevronDown } from 'lucide-react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { VncEmbed } from '@/components/vnc-embed';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { ToolSpec, CommandSpec } from '@/lib/types';

function buildCurlCommand(toolName: string, args: Record<string, unknown>): string {
  const body = JSON.stringify({
    jsonrpc: '2.0',
    id: 'console',
    method: 'tools/call',
    params: { name: toolName, arguments: args },
  });
  return `curl -X POST http://localhost:12233/mcp \\\n  -H 'Content-Type: application/json' \\\n  -H 'Authorization: Bearer <password>' \\\n  -d '${body}'`;
}

export default function McpPage() {
  const { lang } = useLang();
  const tr = t(lang).mcp;
  const [tools, setTools] = useState<ToolSpec[]>([]);
  const [commands, setCommands] = useState<CommandSpec[]>([]);
  const [toolName, setToolName] = useState('');
  const [params, setParams] = useState<Record<string, string>>({});
  const [result, setResult] = useState('');
  const [curlCmd, setCurlCmd] = useState('');
  const [running, setRunning] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [toolRes, cmdRes] = await Promise.all([api.getMcpTools(), api.getCommands()]);
        setTools(toolRes.data);
        setCommands(cmdRes.data);
        if (toolRes.data.length > 0) {
          setToolName(toolRes.data[0].name);
        }
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

  const selectedTool = useMemo(() => tools.find((t) => t.name === toolName), [tools, toolName]);
  const cmdSpec = useMemo(() => {
    if (!selectedTool) return undefined;
    return commands.find((c) => c.name === selectedTool.command);
  }, [commands, selectedTool]);

  const handleToolChange = useCallback((name: string) => {
    setToolName(name);
    setParams({});
    setResult('');
    setCurlCmd('');
  }, []);

  const handleParamChange = (name: string, value: string) => {
    setParams((prev) => ({ ...prev, [name]: value }));
  };

  const parseParams = (): Record<string, unknown> | null => {
    if (!cmdSpec) return {};
    const parsed: Record<string, unknown> = {};
    for (const p of cmdSpec.params) {
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

  const handleCall = async () => {
    const parsed = parseParams();
    if (!parsed) return;

    setCurlCmd(buildCurlCommand(toolName, parsed));
    setRunning(true);
    setResult('');
    try {
      const res = await api.callMcpTool(toolName, parsed);
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
          {/* Left: caller */}
          <Card hover={false}>
            <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.title}</h1>
            <p className="text-sm text-slate-500 mb-4">{tr.description}</p>
            <pre className="mb-3 text-xs">Authorization: Bearer &lt;console-password&gt;</pre>
            <p className="text-sm text-slate-600 mb-1">{tr.endpoint}<code>/mcp</code></p>
            <p className="text-sm text-slate-600 mb-6">{tr.tool_index}<code>/api/mcp/tools</code></p>

            <div className="space-y-4">
              <div>
                <label>{tr.tool_label}</label>
                <div className="relative">
                  <select
                    value={toolName}
                    onChange={(e) => handleToolChange(e.target.value)}
                    className="mt-1 appearance-none pr-10"
                  >
                    {tools.map((tool) => (
                      <option key={tool.name} value={tool.name}>
                        {tool.name} → {tool.command}
                      </option>
                    ))}
                  </select>
                  <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 mt-0.5 h-4 w-4 text-slate-400 pointer-events-none" />
                </div>
              </div>

              {cmdSpec?.params.map((p) => (
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

              <button onClick={handleCall} disabled={running} className="btn-primary">
                {running ? <><Spinner /> {tr.call_tool}</> : tr.call_tool}
              </button>

              {curlCmd && (
                <pre className="mt-4 text-xs whitespace-pre-wrap"><span className="text-slate-400">$ </span>{curlCmd}</pre>
              )}

              {result && <pre className="mt-2 max-h-96 overflow-auto whitespace-pre-wrap break-words">{result}</pre>}

              <VncEmbed />
            </div>
          </Card>

          {/* Right: tool list */}
          <Card hover={false}>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.tools_heading}</h2>
            <ul className="space-y-2">
              {tools.map((tool) => (
                <li key={tool.name} className="flex items-center gap-2 text-sm">
                  <span className="font-semibold text-slate-900">{tool.name}</span>
                  <span className="text-slate-400">&rarr;</span>
                  <code>{tool.command}</code>
                </li>
              ))}
            </ul>
          </Card>
        </div>
      </main>
    </>
  );
}
