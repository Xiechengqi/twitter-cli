'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { RefreshCw } from 'lucide-react';
import { StatusDot } from './status-dot';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AppConfig } from '@/lib/types';

export function VncEmbed() {
  const { lang } = useLang();
  const tr = t(lang).home;
  const [config, setConfig] = useState<AppConfig | null>(null);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getConfig();
        setConfig(res.data);
      } catch { /* ignore */ }
    })();
  }, []);

  const handleRefresh = useCallback(() => {
    const iframe = iframeRef.current;
    if (iframe) {
      iframe.src = iframe.src;
    }
  }, []);

  const configured = config ? !!(config.vnc.url && config.vnc.embed) : false;

  if (!config) return null;

  return (
    <div className="mt-6 pt-6 border-t border-slate-200 dark:border-slate-700">
      <h3 className="text-sm font-bold text-slate-900 dark:text-white mb-3 flex items-center gap-2">
        <StatusDot ok={configured} />
        VNC
        {configured && (
          <button
            onClick={handleRefresh}
            className="p-1 rounded-md text-slate-400 hover:text-brand-600 hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors"
            title="Refresh"
          >
            <RefreshCw className="h-3.5 w-3.5" />
          </button>
        )}
      </h3>
      {configured ? (
        <div className="w-full rounded-xl border border-slate-200 dark:border-slate-700 overflow-hidden aspect-video">
          <iframe
            ref={iframeRef}
            src={config.vnc.url}
            className="w-full h-full border-0 bg-white dark:bg-slate-900"
          />
        </div>
      ) : (
        <p className="text-sm text-slate-500 dark:text-slate-400">{tr.vnc_not_configured}</p>
      )}
    </div>
  );
}
