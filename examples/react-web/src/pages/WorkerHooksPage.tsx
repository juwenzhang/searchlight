import { useState } from 'react';
import { useSearchlightWorker } from '@luhanxin/searchlight-react';
import searchlightWorkerUrl from '@luhanxin/searchlight-react/worker?worker&url';
import { DocumentShelf } from '../components/DocumentShelf';
import { ResultList } from '../components/ResultList';
import { SearchPanel } from '../components/SearchPanel';
import { UsageNotes } from '../components/UsageNotes';
import { documents, toSearchText, workerDocuments, workerExamples, type DemoDocument } from '../data';

const workerHookOptions = {
  initialQuery: 'Worker',
  getText: toSearchText,
  workerUrl: searchlightWorkerUrl,
  relatedLimit: 8,
  searchOptions: {
    fuzzy: true,
    maxEditDistance: 2,
    usePinyin: true,
    highlight: true,
    limit: 10,
    enableCache: true,
    explain: true,
  },
};

const workerHookCode = `import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const search = useSearchlightWorker(documents, {
  initialQuery: 'Worker',
  getText: toSearchText,
  workerUrl,
  searchOptions: { fuzzy: true, usePinyin: true, enableCache: true },
});

await search.reindex(nextDocuments);
await search.search(search.query);`;

function toErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function WorkerHooksPage() {
  const [indexedDocuments, setIndexedDocuments] = useState<DemoDocument[]>(documents);
  const [actionNote, setActionNote] = useState('默认文档已在 Worker 内完成索引。');
  const [pending, setPending] = useState(false);
  const search = useSearchlightWorker(documents, workerHookOptions);

  async function runWorkerAction(label: string, documentsToIndex: DemoDocument[]) {
    setPending(true);
    setActionNote(`${label}中...`);

    try {
      await search.reindex(documentsToIndex);
      setIndexedDocuments(documentsToIndex);
      await search.search(search.query || 'Worker');
      setActionNote(`${label}完成，当前 Worker 索引 ${documentsToIndex.length} 篇文档。`);
    } catch (error: unknown) {
      setActionNote(`${label}失败：${toErrorMessage(error)}`);
    } finally {
      setPending(false);
    }
  }

  async function clearWorkerIndex() {
    setPending(true);
    setActionNote('正在清空 Worker 索引...');

    try {
      await search.clear();
      setIndexedDocuments([]);
      setActionNote('Worker 索引已清空，可点击“恢复默认索引”重新写入文档。');
    } catch (error: unknown) {
      setActionNote(`清空 Worker 索引失败：${toErrorMessage(error)}`);
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="demo-page">
      <SearchPanel
        title="useSearchlightWorker：Worker 异步搜索"
        description="搜索引擎在 Web Worker 内初始化，search、suggest、reindex、clear 都是 Promise，不阻塞页面交互。"
        query={search.query}
        ready={search.ready && !pending}
        status={search.status}
        placeholder="输入：Worker / reindex / suggest / bj / 缓存"
        examples={workerExamples}
        suggestions={search.suggestions}
        relatedSuggestions={search.relatedSuggestions}
        error={search.error}
        actionNote={actionNote}
        onQueryChange={search.setQuery}
        onSearch={() => search.search(search.query)}
        onClear={clearWorkerIndex}
      />

      <section className="panel worker-actions">
        <div>
          <p className="eyebrow">Worker controls</p>
          <h2>动态索引演示</h2>
          <p>点击按钮会调用 Worker hook 暴露的异步 reindex / clear 能力。</p>
        </div>
        <div className="button-row">
          <button
            className="primary-button"
            disabled={!search.ready || pending}
            onClick={() => void runWorkerAction('扩展 Worker 索引', workerDocuments)}
          >
            扩展 Worker 索引
          </button>
          <button
            className="ghost-button"
            disabled={!search.ready || pending}
            onClick={() => void runWorkerAction('恢复默认索引', documents)}
          >
            恢复默认索引
          </button>
        </div>
      </section>

      <div className="content-grid">
        <DocumentShelf
          title="Worker 当前索引"
          description="这里反映最近一次 reindex / clear 后的文档集合。"
          documents={indexedDocuments}
        />
        <ResultList
          title="Worker Hook 搜索结果"
          results={search.results}
          emptyText="暂无结果，试试 Worker、reindex、suggest 或先扩展 Worker 索引。"
        />
      </div>

      <UsageNotes
        title="Worker Hook 关键点"
        points={[
          'useSearchlightWorker 的搜索、建议词、重建索引和清空索引方法都返回 Promise。',
          '默认 Worker URL 由 @luhanxin/searchlight-react 包提供，也可以传入 worker 或 workerUrl 自定义。',
          '适合较大文档集、频繁输入和需要保持主线程流畅的搜索页面。',
          'suggestRelated 会在 Worker 中基于命中文档共现词返回相关提示词。',
        ]}
        code={workerHookCode}
      />
    </div>
  );
}
