'use client';

import { useState } from 'react';
import { Eye, EyeOff } from 'lucide-react';

type PasswordInputProps = {
  id?: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  autoComplete?: string;
  showLabel: string;
  hideLabel: string;
  className?: string;
};

export function PasswordInput({
  id,
  label,
  value,
  onChange,
  autoComplete,
  showLabel,
  hideLabel,
  className,
}: PasswordInputProps) {
  const [visible, setVisible] = useState(false);

  return (
    <div className={className}>
      <label htmlFor={id}>{label}</label>
      <div className="mt-1 flex items-center gap-2">
        <input
          id={id}
          type={visible ? 'text' : 'password'}
          autoComplete={autoComplete}
          value={value}
          onChange={(e) => onChange(e.target.value)}
        />
        <button
          type="button"
          className="btn-secondary shrink-0 px-3 py-2"
          onClick={() => setVisible((current) => (current ? false : true))}
          aria-label={visible ? hideLabel : showLabel}
        >
          {visible ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          <span>{visible ? hideLabel : showLabel}</span>
        </button>
      </div>
    </div>
  );
}
