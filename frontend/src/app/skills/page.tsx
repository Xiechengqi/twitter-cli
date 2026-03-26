'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { SkillSpec } from '@/lib/types';

export default function SkillsPage() {
  const { lang } = useLang();
  const tr = t(lang).skills;
  const [skills, setSkills] = useState<SkillSpec[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getSkills();
        setSkills(res.data);
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 flex justify-center"><Spinner /></main>
      </>
    );
  }

  const skillDocs: Record<string, { description: string; steps: { step: string; command: string; params: string }[]; example: string }> = {
    research_account: {
      description: lang === 'zh'
        ? '分析目标账号的档案、时间线和关注列表。适用于竞争分析、社交关系研究或尽职调查场景。'
        : 'Analyze a target account\'s profile, timeline, and followings. Useful for competitive analysis, social graph research, or due diligence.',
      steps: [
        { step: '1', command: 'get_user_by_username', params: 'username' },
        { step: '2', command: 'get_user_timeline', params: 'user_id, limit' },
        { step: '3', command: 'get_followers', params: 'user_id, limit' },
      ],
      example: lang === 'zh'
        ? '/research_account @username\n→ 获取账号信息、时间线推文、粉丝列表'
        : '/research_account @username\n→ Get account info, timeline tweets, follower list',
    },
    monitor_keyword: {
      description: lang === 'zh'
        ? '持续监控关键词和趋势活动。适用于品牌声誉管理、竞品追踪或事件监测。'
        : 'Monitor keyword and trend activity over time. Useful for brand reputation management, competitor tracking, or event monitoring.',
      steps: [
        { step: '1', command: 'search_tweets', params: 'query, count' },
        { step: '2', command: 'get_trending', params: '-' },
        { step: '3', command: 'search_tweets', params: 'query, count (repeat)' },
      ],
      example: lang === 'zh'
        ? '/monitor_keyword "twitter-cli"\n→ 搜索相关推文并获取热门趋势'
        : '/monitor_keyword "twitter-cli"\n→ Search related tweets and get trending topics',
    },
    prepare_reply_context: {
      description: lang === 'zh'
        ? '在撰写回复前收集上下文。获取原始推文、作者信息和最近的对话线程。'
        : 'Collect context before composing a reply. Fetch the original tweet, author info, and recent conversation thread.',
      steps: [
        { step: '1', command: 'get_user_by_username', params: 'username' },
        { step: '2', command: 'get_tweet_detail', params: 'tweet_id' },
        { step: '3', command: 'search_tweets', params: 'query (thread context)' },
      ],
      example: lang === 'zh'
        ? '/prepare_reply_context tweet_id\n→ 获取推文详情、作者资料、线程上下文'
        : '/prepare_reply_context tweet_id\n→ Get tweet detail, author profile, thread context',
    },
  };

  return (
    <>
      <Nav authenticated />
      <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 space-y-8">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.skills_title}</h1>
          <p className="text-sm text-slate-500">{tr.skills_description}</p>
        </div>

        {skills.map((skill) => {
          const doc = skillDocs[skill.name];
          return (
            <Card key={skill.name} hover={false}>
              <div className="mb-4">
                <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
                  <span className="text-brand-600">{skill.name}</span>
                  {skill.requires_auth && (
                    <span className="text-xs px-1.5 py-0.5 rounded-full bg-amber-50 text-amber-600">auth</span>
                  )}
                </h2>
                <p className="text-sm text-slate-500 mt-1">{doc?.description || skill.summary}</p>
              </div>

              {doc && (
                <>
                  <div className="mb-4">
                    <h3 className="text-sm font-semibold text-slate-700 mb-2">{lang === 'zh' ? '执行步骤' : 'Execution Steps'}</h3>
                    <div className="space-y-2">
                      {doc.steps.map((s, i) => (
                        <div key={i} className="flex items-start gap-3 text-sm">
                          <span className="w-6 h-6 rounded-full bg-brand-50 text-brand-600 flex items-center justify-center text-xs font-bold flex-shrink-0 mt-0.5">
                            {s.step}
                          </span>
                          <div>
                            <code className="text-brand-600">{s.command}</code>
                            {s.params !== '-' && <span className="text-slate-400 ml-1">({s.params})</span>}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="bg-slate-50 rounded-lg p-3 text-sm">
                    <span className="text-slate-400 font-mono">{doc.example}</span>
                  </div>
                </>
              )}
            </Card>
          );
        })}
      </main>
    </>
  );
}
