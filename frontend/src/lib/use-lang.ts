'use client';

import { useCallback, useEffect, useState } from 'react';
import type { Lang } from './i18n';

export function useLang() {
  const [lang, setLangState] = useState<Lang>('en');

  useEffect(() => {
    const stored = localStorage.getItem('lang') as Lang | null;
    if (stored === 'zh' || stored === 'en') setLangState(stored);
  }, []);

  const setLang = useCallback((next: Lang) => {
    setLangState(next);
    localStorage.setItem('lang', next);
    document.cookie = `lang=${next};path=/;max-age=31536000`;
  }, []);

  const toggle = useCallback(() => {
    setLang(lang === 'en' ? 'zh' : 'en');
  }, [lang, setLang]);

  return { lang, setLang, toggle };
}
