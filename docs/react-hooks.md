# Searchlight React Hooks 完整指南

`@luhanxin/searchlight-react` 提供 React 侧的工具层封装：

- `useSearchlight`：主线程 WASM 搜索。
- `useSearchlightWorker`：Web Worker 异步搜索。
- `useSearchlightEngine`：直接获取原始 WASM `SearchEngine`。
- `LocalSearchProvider` / `WorkerSearchProvider`：非 React hook 场景的 provider。
- `WorkerSearchClient`：更底层的 Worker RPC client。

## 什么时候用哪个 API

| 场景 | 推荐 API |
| --- | --- |
| 快速在 React 文档站里加搜索 | `useSearchlight` |
| 文档多、输入频繁、避免卡 UI | `useSearchlightWorker` |
| 想自己控制索引和搜索生命周期 | `LocalSearchProvider` / `WorkerSearchProvider` |
| 想直接调用 WASM 原始方法 | `useSearchlightEngine` |
| 非 React、只想用 Worker RPC | `WorkerSearchClient` |

## 基础文档类型

```ts
type Doc = {
  id: string;
  title: string;
  summary: string;
  content: string;
  route: string;
  tags: string[];
};

function toSearchText(doc: Doc) {
  return [doc.title, doc.summary, doc.content, doc.route, doc.tags.join(' ')].join('\n');
}
```

`searchlight` 核心只认识字符串文档，React 封装通过 `getText` 建索引，并把结果的 `doc_id` 映射回原始 `Doc`，放到 `hit.item`。

## `useSearchlight`

```tsx
import { useSearchlight } from '@luhanxin/searchlight-react';

export function LocalSearch({ docs }: { docs: Doc[] }) {
  const search = useSearchlight(docs, {
    initialQuery: '本地搜索',
    getText: toSearchText,
    relatedLimit: 8,
    searchOptions: {
      fuzzy: true,
      maxEditDistance: 2,
      usePinyin: true,
      highlight: true,
      explain: true,
      limit: 10,
    },
  });

  return (
    <section>
      <input value={search.query} disabled={!search.ready} onChange={(event) => search.setQuery(event.target.value)} />
      {search.results.map((hit) => <article key={hit.doc_id}>{hit.item?.title}</article>)}
    </section>
  );
}
```

### 自动行为

默认情况下：

- `autoSearch: true`：`query` 变化后自动执行 `search(query)`。
- `suggest: true`：自动刷新 `suggestions`。
- `related: true`：自动刷新 `relatedSuggestions`。
- `relatedLimit: 8`：最多返回 8 个相关提示词。

如果你要自己控制触发时机：

```ts
const search = useSearchlight(docs, {
  autoSearch: false,
  suggest: false,
  related: false,
  getText: toSearchText,
});

const results = search.search('worker', { limit: 5 });
const suggestions = search.suggest('wo');
const related = search.suggestRelated('worker search', 5);
```

## `useSearchlightWorker`

```tsx
import { useSearchlightWorker } from '@luhanxin/searchlight-react';
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

export function WorkerSearch({ docs }: { docs: Doc[] }) {
  const search = useSearchlightWorker(docs, {
    initialQuery: 'Worker',
    getText: toSearchText,
    workerUrl,
    relatedLimit: 8,
    searchOptions: {
      fuzzy: true,
      usePinyin: true,
      highlight: true,
      enableCache: true,
      explain: true,
    },
  });

  async function refresh(nextDocs: Doc[]) {
    await search.reindex(nextDocs);
    await search.search(search.query);
    await search.suggestRelated(search.query);
  }

  return null;
}
```

Worker hook 和主线程 hook 返回字段基本一致，但方法是异步的：

```ts
await search.search('worker');
await search.suggest('wo');
await search.suggestRelated('worker search', 8);
await search.reindex(nextDocs);
await search.clear();
```

## 搜索补全 vs 相关提示词

### `suggest(prefix)`

输入前缀，返回索引词表中的补全词：

```ts
search.suggest('pro');
search.suggest('bj'); // 拼音/首字母
```

适合输入框 autocomplete。

### `suggestRelated(query, limit)`

输入完整 query，先搜索 top 文档，再从命中文档里召回共现词：

```ts
search.suggestRelated('react search', 8);
```

适合：

- 相关搜索
- 下一步搜索提示词
- “你可能还想搜”
- AI/RAG 中 topK 召回后的二次检索入口

它不是 LLM 生成，不会凭空编问题。

## Explain 结果展示

开启：

```ts
const results = search.search('worker search', {
  explain: true,
  highlight: true,
});
```

读取：

```ts
const first = results[0];
first.score_breakdown?.bm25;
first.score_breakdown?.proximity;
first.score_breakdown?.coverage;
first.score_breakdown?.phrase;
first.score_breakdown?.pinyin;
first.match_reasons?.map((reason) => reason.message);
```

## Provider 用法

### LocalSearchProvider

```ts
import { LocalSearchProvider } from '@luhanxin/searchlight-react';

const provider = new LocalSearchProvider<Doc>({
  documents: docs,
  getText: toSearchText,
  bm25: { k1: 1.2, b: 0.75 },
  searchOptions: { fuzzy: true, usePinyin: true, highlight: true },
});

await provider.init();
provider.search('React');
provider.batchSearch(['React', 'Worker']);
provider.suggest('re');
provider.suggestRelated('react search', 8);
provider.reindex(nextDocs);
provider.clear();
provider.dispose();
```

### WorkerSearchProvider

```ts
import { WorkerSearchProvider } from '@luhanxin/searchlight-react';
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const provider = new WorkerSearchProvider<Doc>({
  documents: docs,
  getText: toSearchText,
  workerUrl,
});

await provider.init();
await provider.search('Worker');
await provider.suggest('wo');
await provider.suggestRelated('worker search', 8);
await provider.reindex(nextDocs);
await provider.clear();
provider.dispose();
```

## Worker 打包说明

Vite 推荐：

```ts
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';
```

然后传给 hook/provider：

```ts
useSearchlightWorker(docs, { workerUrl, getText: toSearchText });
```

自定义 Worker：

```ts
const worker = new Worker(new URL('./my-search.worker.ts', import.meta.url), { type: 'module' });
useSearchlightWorker(docs, { worker, getText: toSearchText });
```

自定义 WASM URL：

```ts
useSearchlightWorker(docs, {
  workerUrl,
  wasmModuleUrl: '/assets/searchlight_bg.wasm',
  getText: toSearchText,
});
```

## 类型速查

```ts
type SearchlightSearchOptions = {
  fuzzy?: boolean;
  maxEditDistance?: number;
  usePinyin?: boolean;
  highlight?: boolean;
  limit?: number;
  enableCache?: boolean;
  explain?: boolean;
};
```

```ts
type SearchlightRelatedSuggestion = {
  term: string;
  score: number;
  doc_frequency: number;
  total_frequency: number;
  source_doc_ids: number[];
};
```

```ts
type UseSearchlightOptions<T> = {
  getText?: (document: T, index: number) => string;
  wasmModule?: InitInput | Promise<InitInput>;
  bm25?: { k1: number; b: number };
  searchOptions?: SearchlightSearchOptions;
  initialQuery?: string;
  autoSearch?: boolean;
  suggest?: boolean;
  related?: boolean;
  relatedLimit?: number;
};
```

## 常见坑

- `options` 对象建议放在组件外，或用 `useMemo`，避免重复初始化索引。
- `snippet` 含 HTML，高亮展示前请确认文档可信或做清洗。
- Worker API 是异步的，按钮事件里建议 `void search.search(...)` 或 `await`。
- 前端本地索引不适合敏感全文数据。
- 大文档库优先使用 Worker 或后端搜索。
