'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { CommandSpec, SkillSpec } from '@/lib/types';

export default function DocsPage() {
  const { lang } = useLang();
  const tr = t(lang).docs;
  const [commands, setCommands] = useState<CommandSpec[]>([]);
  const [skills, setSkills] = useState<SkillSpec[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [cmdRes, skillRes] = await Promise.all([api.getCommands(), api.getSkills()]);
        setCommands(cmdRes.data);
        setSkills(skillRes.data);
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

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
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 space-y-6">
        <div className="mb-6">
          <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.title}</h1>
          <p className="text-sm text-slate-500">{tr.description}</p>
        </div>

        {/* Commands Table */}
        <Card hover={false}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm text-left">
              <thead>
                <tr className="border-b border-slate-200">
                  <th className="py-2 pr-4 font-semibold text-slate-700">{tr.command}</th>
                  <th className="py-2 pr-4 font-semibold text-slate-700">{tr.category}</th>
                  <th className="py-2 pr-4 font-semibold text-slate-700">{tr.mode}</th>
                  <th className="py-2 font-semibold text-slate-700">{tr.summary}</th>
                </tr>
              </thead>
              <tbody>
                {commands.map((c) => (
                  <tr key={c.name} className="border-b border-slate-100 last:border-0">
                    <td className="py-2 pr-4 font-semibold text-slate-900">{c.name}</td>
                    <td className="py-2 pr-4">
                      <span className={`text-xs px-1.5 py-0.5 rounded-full ${
                        c.category === 'read'
                          ? 'bg-emerald-50 text-emerald-600'
                          : 'bg-amber-50 text-amber-600'
                      }`}>{c.category}</span>
                    </td>
                    <td className="py-2 pr-4 text-slate-600">{c.execution_mode}</td>
                    <td className="py-2 text-slate-600">{c.summary}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        {/* Skills */}
        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.skills}</h2>
          <ul className="space-y-3">
            {skills.map((s) => (
              <li key={s.name}>
                <span className="font-semibold text-sm text-slate-900">{s.name}</span>
                <span className="text-sm text-slate-500 ml-2">{s.summary}</span>
              </li>
            ))}
          </ul>
        </Card>
      </main>
    </>
  );
}
