'use client';

import { useEffect, useState } from 'react';
import { StatusDot } from './status-dot';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AppConfig } from '@/lib/types';

export function VncEmbed() {
  const { lang } = useLang();
  const tr = t(lang).home;
  const [config, setConfig] = useState<AppConfig | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getConfig();
        setConfig(res.data);
      } catch { /* ignore */ }
    })();
  }, []);

  const configured = config ? !!(config.vnc.url && config.vnc.embed) : false;

  if (!config) return null;

  return (
    <div className="mt-6 pt-6 border-t border-slate-200 dark:border-slate-700">
      <h3 className="text-sm font-bold text-slate-900 dark:text-white mb-3 flex items-center gap-2">
        <StatusDot ok={configured} />
        VNC
      </h3>
      {configured ? (
        <iframe
          src={config.vnc.url}
          className="w-full h-80 border border-slate-200 dark:border-slate-700 rounded-xl bg-white dark:bg-slate-900"
        />
      ) : (
        <p className="text-sm text-slate-500 dark:text-slate-400">{tr.vnc_not_configured}</p>
      )}
    </div>
  );
}
