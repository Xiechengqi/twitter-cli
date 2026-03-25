'use client';

import { useEffect, useRef, useState } from 'react';
import { StatusDot } from './status-dot';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AppConfig } from '@/lib/types';

const VNC_W = 1920;
const VNC_H = 1080;

export function VncEmbed() {
  const { lang } = useLang();
  const tr = t(lang).home;
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [scale, setScale] = useState(1);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getConfig();
        setConfig(res.data);
      } catch { /* ignore */ }
    })();
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver(([entry]) => {
      setScale(entry.contentRect.width / VNC_W);
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [config]);

  const configured = config ? !!(config.vnc.url && config.vnc.embed) : false;

  if (!config) return null;

  return (
    <div className="mt-6 pt-6 border-t border-slate-200 dark:border-slate-700">
      <h3 className="text-sm font-bold text-slate-900 dark:text-white mb-3 flex items-center gap-2">
        <StatusDot ok={configured} />
        VNC
      </h3>
      {configured ? (
        <div
          ref={containerRef}
          className="w-full rounded-xl border border-slate-200 dark:border-slate-700 overflow-hidden"
          style={{ height: VNC_H * scale }}
        >
          <iframe
            src={config.vnc.url}
            className="border-0 bg-white dark:bg-slate-900"
            style={{
              width: VNC_W,
              height: VNC_H,
              transform: `scale(${scale})`,
              transformOrigin: 'top left',
            }}
          />
        </div>
      ) : (
        <p className="text-sm text-slate-500 dark:text-slate-400">{tr.vnc_not_configured}</p>
      )}
    </div>
  );
}
