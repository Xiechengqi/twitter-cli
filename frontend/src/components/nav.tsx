'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Sun, Moon, Monitor, Globe, LogOut } from 'lucide-react';
import { clsx } from 'clsx';
import { useTheme } from '@/lib/use-theme';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';

const themeIcon = { auto: Monitor, dark: Moon, light: Sun } as const;
const themeLabel = { auto: 'Auto', dark: 'Dark', light: 'Light' } as const;

export function Nav({ authenticated }: { authenticated: boolean }) {
  const pathname = usePathname();
  const { theme, cycle } = useTheme();
  const { lang, toggle: toggleLang } = useLang();
  const tr = t(lang);
  const ThemeIcon = themeIcon[theme];

  const navLink = (href: string, label: string) => (
    <Link
      href={href}
      className={clsx(
        'px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200',
        pathname === href
          ? 'bg-brand-50 dark:bg-brand-950 text-brand-600 dark:text-brand-400'
          : 'text-slate-600 dark:text-slate-300 hover:bg-brand-50 dark:hover:bg-brand-950 hover:text-brand-600 dark:hover:text-brand-400',
      )}
    >
      {label}
    </Link>
  );

  const handleLogout = async () => {
    await api.logout();
    window.location.href = '/login';
  };

  return (
    <nav className="sticky top-0 z-50 bg-white/85 dark:bg-slate-900/85 backdrop-blur-xl border-b border-slate-200 dark:border-slate-800">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 flex items-center justify-between h-14">
        <div className="flex items-center gap-1">
          <Link href="/" className="text-lg font-extrabold gradient-text mr-4">
            twitter-cli
          </Link>
          {authenticated ? (
            <>
              {navLink('/', tr.nav.console)}
              {navLink('/commands', tr.nav.commands)}
              {navLink('/mcp', tr.nav.mcp)}
              {navLink('/docs', tr.nav.docs)}
              {navLink('/settings', tr.nav.settings)}
            </>
          ) : (
            <>
              {navLink('/login', tr.nav.login)}
              {navLink('/setup/password', tr.nav.setup_password)}
            </>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={cycle}
            className="p-2 rounded-lg text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            title={themeLabel[theme]}
          >
            <ThemeIcon className="h-4 w-4" />
          </button>
          <button
            onClick={toggleLang}
            className="p-2 rounded-lg text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-xs font-semibold"
          >
            <Globe className="h-4 w-4" />
          </button>
          {authenticated && (
            <button
              onClick={handleLogout}
              className="p-2 rounded-lg text-slate-500 dark:text-slate-400 hover:bg-red-50 dark:hover:bg-red-950 hover:text-red-600 transition-colors"
              title={tr.nav.logout}
            >
              <LogOut className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>
    </nav>
  );
}
