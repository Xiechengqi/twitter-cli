'use client';

import { useEffect, useState, useCallback } from 'react';
import { X, ZoomIn } from 'lucide-react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AccountEntry, PreviewPost } from '@/lib/types';

function ImageLightbox({ src, onClose }: { src: string; onClose: () => void }) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
      onClick={onClose}
    >
      <button
        onClick={onClose}
        className="absolute top-4 right-4 p-2 rounded-full bg-white/10 text-white hover:bg-white/20 transition-colors"
      >
        <X className="h-5 w-5" />
      </button>
      <img
        src={src}
        alt=""
        onClick={(e) => e.stopPropagation()}
        className="max-h-[90vh] max-w-[90vw] rounded-xl shadow-2xl object-contain"
      />
    </div>
  );
}

function formatTimestamp(epoch: number, lang: 'en' | 'zh'): string {
  const tr = t(lang).components;
  const now = Date.now() / 1000;
  const diff = Math.max(0, now - epoch);
  if (diff < 60) return tr.just_now;
  if (diff < 3600) return `${Math.floor(diff / 60)}${tr.minutes_ago}`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}${tr.hours_ago}`;
  return `${Math.floor(diff / 86400)}${tr.days_ago}`;
}

function PreviewCard({
  post,
  accounts,
  onSent,
  onDeleted,
}: {
  post: PreviewPost;
  accounts: AccountEntry[];
  onSent: (id: string) => void;
  onDeleted: (id: string) => void;
}) {
  const { lang } = useLang();
  const tr = t(lang).preview;
  const [content, setContent] = useState(post.content);
  const [dirty, setDirty] = useState(false);
  const [sending, setSending] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [sent, setSent] = useState(false);
  const [lightbox, setLightbox] = useState(false);

  const account = accounts.find((a) => a.cdp_port === post.cdp_port);
  const accountLabel = account?.username ? `@${account.username}` : `port ${post.cdp_port}`;

  const handleBlur = useCallback(async () => {
    if (!dirty) return;
    await api.updatePreviewPost(post.id, content, post.image);
    setDirty(false);
  }, [dirty, content, post.id, post.image]);

  const handleSend = async () => {
    setSending(true);
    try {
      // Persist any unsaved edits first
      if (dirty) {
        await api.updatePreviewPost(post.id, content, post.image);
        setDirty(false);
      }
      await api.sendPreviewPost(post.id);
      setSent(true);
      setTimeout(() => onSent(post.id), 800);
    } finally {
      setSending(false);
    }
  };

  const handleDelete = async () => {
    setDeleting(true);
    try {
      await api.deletePreviewPost(post.id);
      onDeleted(post.id);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <Card hover={false}>
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          {account?.avatar_url ? (
            <img src={account.avatar_url} alt="" className="h-7 w-7 rounded-full border border-slate-200" />
          ) : (
            <div className="h-7 w-7 rounded-full bg-indigo-100 flex items-center justify-center text-xs font-bold text-indigo-600">
              {accountLabel.charAt(1)?.toUpperCase() || '?'}
            </div>
          )}
          <span className="text-sm font-semibold text-slate-700">{accountLabel}</span>
        </div>
        <span className="text-xs text-slate-400">{formatTimestamp(post.created_at, lang)}</span>
      </div>

      <textarea
        rows={3}
        value={content}
        placeholder={tr.edit_placeholder}
        onChange={(e) => { setContent(e.target.value); setDirty(true); }}
        onBlur={handleBlur}
        className="mt-1 w-full resize-none"
      />

      {post.image && (() => {
        const imgSrc = `/api/uploads/${encodeURIComponent(post.image.split('/').pop() ?? '')}`;
        return (
          <div className="mt-3">
            <div className="relative inline-block group cursor-zoom-in" onClick={() => setLightbox(true)}>
              <img
                src={imgSrc}
                alt=""
                className="h-24 w-24 rounded-lg object-cover border border-slate-200 shadow-sm transition-opacity group-hover:opacity-80"
              />
              <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                <ZoomIn className="h-5 w-5 text-white drop-shadow-md" />
              </div>
            </div>
            <p className="text-xs text-slate-400 mt-1 truncate max-w-xs">{post.image}</p>
            {lightbox && <ImageLightbox src={imgSrc} onClose={() => setLightbox(false)} />}
          </div>
        );
      })()}

      <div className="mt-4 flex items-center gap-3">
        <button
          onClick={handleSend}
          disabled={sending || sent}
          className="btn-primary text-sm py-1.5 px-4"
        >
          {sent ? tr.posted : sending ? <><Spinner /> {tr.sending}</> : tr.send}
        </button>
        <button
          onClick={handleDelete}
          disabled={deleting || sending}
          className="text-sm text-slate-500 hover:text-red-600 transition-colors font-medium disabled:opacity-50"
        >
          {deleting ? <Spinner /> : tr.delete}
        </button>
      </div>
    </Card>
  );
}

export default function PreviewPage() {
  const { lang } = useLang();
  const tr = t(lang).preview;
  const [posts, setPosts] = useState<PreviewPost[]>([]);
  const [accounts, setAccounts] = useState<AccountEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [postsRes, accRes] = await Promise.all([api.getPreviewPosts(), api.getAccounts()]);
        setPosts(postsRes.data);
        setAccounts(accRes.data);
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

  const remove = useCallback((id: string) => {
    setPosts((prev) => prev.filter((p) => p.id !== id));
  }, []);

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-2xl mx-auto px-4 sm:px-6 py-16 flex justify-center"><Spinner /></main>
      </>
    );
  }

  return (
    <>
      <Nav authenticated />
      <main className="max-w-2xl mx-auto px-4 sm:px-6 py-16">
        <h1 className="text-2xl font-bold text-slate-900 mb-1">{tr.title}</h1>
        <p className="text-sm text-slate-500 mb-8">{tr.description}</p>

        {posts.length === 0 ? (
          <Card hover={false}>
            <p className="text-sm text-slate-500 text-center py-4">{tr.empty}</p>
          </Card>
        ) : (
          <div className="space-y-4">
            {posts.map((post) => (
              <PreviewCard
                key={post.id}
                post={post}
                accounts={accounts}
                onSent={remove}
                onDeleted={remove}
              />
            ))}
          </div>
        )}
      </main>
    </>
  );
}
