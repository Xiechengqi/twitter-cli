import { clsx } from 'clsx';
import type { ReactNode } from 'react';

export function Card({
  children,
  className,
  hover = true,
}: {
  children: ReactNode;
  className?: string;
  hover?: boolean;
}) {
  return (
    <div
      className={clsx(
        'bg-white dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-xl shadow-card p-6',
        hover && 'transition-all duration-200 hover:-translate-y-0.5 hover:shadow-card-hover',
        className,
      )}
    >
      {children}
    </div>
  );
}
