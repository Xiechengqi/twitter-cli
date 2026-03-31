'use client';

import { useCallback, useRef, useState } from 'react';
import { Spinner } from './spinner';
import * as api from '@/lib/api';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';

const ACCEPT = 'image/png,image/jpeg,image/gif,image/webp';

export function FileUploadInput({
  value,
  onChange,
}: {
  value: string;
  onChange: (serverPath: string) => void;
}) {
  const { lang } = useLang();
  const tr = t(lang).upload;
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragging, setDragging] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);

  const upload = useCallback(
    async (file: File) => {
      setUploading(true);
      setPreview(URL.createObjectURL(file));
      setFileName(file.name);
      try {
        const res = await api.uploadFile(file);
        onChange(res.data.path);
      } catch {
        setPreview(null);
        setFileName(null);
      } finally {
        setUploading(false);
      }
    },
    [onChange],
  );

  const handleFiles = useCallback(
    (files: FileList | null) => {
      const file = files?.[0];
      if (file && file.type.startsWith('image/')) upload(file);
    },
    [upload],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragging(false);
      handleFiles(e.dataTransfer.files);
    },
    [handleFiles],
  );

  const hasFile = !!value;

  return (
    <div className="mt-1">
      <div
        onDragOver={(e) => { e.preventDefault(); setDragging(true); }}
        onDragLeave={() => setDragging(false)}
        onDrop={handleDrop}
        onClick={() => inputRef.current?.click()}
        className={`
          relative cursor-pointer rounded-xl border-2 border-dashed transition-all duration-200
          ${dragging
            ? 'border-indigo-500 bg-indigo-50/60'
            : hasFile
              ? 'border-slate-200 bg-slate-50/50'
              : 'border-slate-200 bg-white hover:border-indigo-400 hover:bg-indigo-50/30'
          }
        `}
      >
        <input
          ref={inputRef}
          type="file"
          accept={ACCEPT}
          className="hidden"
          onChange={(e) => handleFiles(e.target.files)}
        />

        {uploading ? (
          <div className="flex items-center justify-center gap-2 py-8">
            <Spinner />
            <span className="text-sm text-slate-500">{tr.uploading}</span>
          </div>
        ) : hasFile && preview ? (
          <div className="flex items-center gap-4 p-4">
            <img
              src={preview}
              alt=""
              className="h-16 w-16 rounded-lg object-cover border border-slate-200 shadow-sm"
            />
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium text-slate-700 truncate">{fileName}</p>
              <p className="text-xs text-slate-400 truncate mt-0.5">{value}</p>
              <button
                type="button"
                onClick={(e) => { e.stopPropagation(); onChange(''); setPreview(null); setFileName(null); }}
                className="mt-1 text-xs text-indigo-600 hover:text-indigo-800 font-medium transition-colors"
              >
                {tr.change}
              </button>
            </div>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 gap-1.5">
            <svg className="h-8 w-8 text-slate-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.41a2.25 2.25 0 013.182 0l2.909 2.91m-18 3.75h16.5a1.5 1.5 0 001.5-1.5V6a1.5 1.5 0 00-1.5-1.5H3.75A1.5 1.5 0 002.25 6v12a1.5 1.5 0 001.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" />
            </svg>
            <p className="text-sm text-slate-500">{tr.drop_hint}</p>
            <p className="text-xs text-slate-400">{tr.formats}</p>
          </div>
        )}
      </div>
    </div>
  );
}
