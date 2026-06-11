# @luhanxin/searchlight-react

`@luhanxin/searchlight-react` 是 `@luhanxin/searchlight` WASM 包的 React 封装，提供主线程搜索、Web Worker 搜索、搜索补全、相关提示词、动态重建索引和状态管理。

它仍然是工具层：只负责把 Rust/WASM 搜索能力变成 React API，不负责 UI、远程抓取、LLM 调用或业务推荐策略。

## 安装

```bash
pnpm add @luhanxin/searchlight @luhanxin/searchlight-react
```

本仓库本地开发：

```bash
pnpm install
pnpm build:wasm
pnpm build:react-hooks
```

## 导出内容

```ts
import {
  initSearchlight,
  createSearchlightEngine,
  LocalSearchProvider,
  WorkerSearchProvider,
  WorkerSearchClient,
  createWorkerSearchClient,
  defaultWorkerUrl,
  useSearchlightEngine,
  useSearchlight,
  useSearchlightWorker,
} from '@luhanxin/searchlight-react';
```

常用类型：

```ts
import type {
  SearchEngine,
  SearchlightSearchOptions,
  SearchlightResult,
  SearchlightHit,
  SearchlightRelatedSuggestion,
  SearchlightProviderOptions,
  SearchlightBm25Options,
  UseSearchlightOptions,
  UseSearchlightState,
  UseSearchlightWorkerOptions,
  UseSearchlightWorkerState,
} from '@luhanxin/searchlight-react';
```

## 推荐使用：`useSearchlight`

适合文档量可控、希望最快接入 React 本地搜索的场景。

```tsx
import { useSearchlight } from '@luhanxin/searchlight-react';

type Doc = {
  id: string;
  title: string;
  summary: string;
  content: string;
  route: string;
  tags: string[];
};

const docs: Doc[] = [
  {
    id: 'react-hooks',
    title: 'React Hooks 本地搜索',
    summary: '通过 useSearchlight 初始化 WASM 索引。',
    content: '支持拼音、模糊搜索、高亮、Explain 和相关提示词。',
    route: '/docs/react-hooks',
    tags: ['React', 'WASM'],
  },
];

function toSearchText(doc: Doc) {
  return [doc.title, doc.summary, doc.content, doc.tags.join(' ')].join('\n');
}

export function SearchBox() {
  const search = useSearchlight(docs, {
    initialQuery: 'React',
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
      <input
        disabled={!search.ready}
        value={search.query}
        onChange={(event) => search.setQuery(event.target.value)}
      />

      {search.error && <p>{search.error.message}</p>}

      <div>
        {search.suggestions.map((item) => (
          <button key={item} onClick={() => search.setQuery(item)}>{item}</button>
        ))}
      </div>

      <div>
        {search.relatedSuggestions.map((item) => (
          <button key={item.term} onClick={() => search.setQuery(item.term)}>
            {item.term}
          </button>
        ))}
      </div>

      {search.results.map((item) => (
        <article key={item.doc_id}>
          <h3>{item.item?.title}</h3>
          <p dangerouslySetInnerHTML={{ __html: item.snippet ?? item.document }} />
        </article>
      ))}
    </section>
  );
}
```

`results` 中会保留原始 WASM 结果，并额外挂载 `item`：

```ts
item.doc_id; // WASM 索引中的数字 ID
item.item;   // 原始 Doc 对象
```

## 大文档/高频输入：`useSearchlightWorker`

Worker 版本把 WASM 引擎放到 Web Worker 里，`search/suggest/suggestRelated/reindex/clear` 都是异步方法。

Vite 推荐显式导入 worker URL：

```tsx
import { useSearchlightWorker } from '@luhanxin/searchlight-react';
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const search = useSearchlightWorker(docs, {
  initialQuery: 'Worker',
  getText: toSearchText,
  workerUrl,
  relatedLimit: 8,
  searchOptions: {
    fuzzy: true,
    maxEditDistance: 2,
    usePinyin: true,
    highlight: true,
    enableCache: true,
    explain: true,
  },
});

await search.search(search.query);
await search.suggest(search.query);
await search.suggestRelated(search.query, 8);
await search.reindex(nextDocs);
await search.clear();
```

如果你的构建工具能正确处理包内 worker，也可以不传 `workerUrl`，由 `defaultWorkerUrl()` 生成默认 URL。

## 低层 API：`useSearchlightEngine`

当你希望直接操作原始 `SearchEngine` 时使用：

```tsx
import { useEffect } from 'react';
import { useSearchlightEngine } from '@luhanxin/searchlight-react';

export function RawEngineDemo() {
  const { ready, engine } = useSearchlightEngine();

  useEffect(() => {
    if (!ready || !engine) return;
    engine.indexBatch(['Rust search', 'React WASM search']);
    const results = engine.searchWithOptions('react', { highlight: true });
    console.log(results);
  }, [ready, engine]);

  return null;
}
```

## Provider API

如果你不想使用 hooks，可以直接使用 provider。

### `LocalSearchProvider`

```ts
import { LocalSearchProvider } from '@luhanxin/searchlight-react';

const provider = new LocalSearchProvider({
  documents: docs,
  getText: toSearchText,
  searchOptions: { fuzzy: true, usePinyin: true, highlight: true },
});

await provider.init();
const results = provider.search('本地搜索');
const suggestions = provider.suggest('bj');
const related = provider.suggestRelated('react search', 8);
provider.reindex(nextDocs);
provider.clear();
provider.dispose();
```

### `WorkerSearchProvider`

```ts
import { WorkerSearchProvider } from '@luhanxin/searchlight-react';
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const provider = new WorkerSearchProvider({
  documents: docs,
  getText: toSearchText,
  workerUrl,
});

await provider.init();
const results = await provider.search('worker');
const suggestions = await provider.suggest('wo');
const related = await provider.suggestRelated('worker search', 8);
await provider.reindex(nextDocs);
await provider.clear();
provider.dispose();
```

## Worker Client API

更底层的 RPC client：

```ts
import { createWorkerSearchClient } from '@luhanxin/searchlight-react';
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const client = createWorkerSearchClient({ workerUrl });
await client.init({
  documents: docs.map(toSearchText),
  searchOptions: { fuzzy: true, usePinyin: true },
});

await client.search('worker');
await client.suggest('bj');
await client.suggestRelated('worker search', 8);
await client.reindex(nextDocs.map(toSearchText));
await client.clear();
client.dispose();
```

## Options

### `SearchlightSearchOptions`

| 字段 | 默认值 | 说明 |
| --- | --- | --- |
| `fuzzy` | `true` in hooks | 开启模糊搜索 |
| `maxEditDistance` | `2` | 最大编辑距离 |
| `usePinyin` | `true` in hooks | 开启拼音/首字母搜索 |
| `highlight` | `true` in hooks | 返回高亮 `snippet` |
| `limit` | `20` | 结果数量 |
| `enableCache` | `true` | 重复 query 缓存 |
| `explain` | `false` | 返回评分拆解与命中原因 |

### `UseSearchlightOptions`

| 字段 | 说明 |
| --- | --- |
| `documents` | provider 构造时可传，hook 通过第一个参数传 |
| `getText` | 把业务文档映射为可索引文本 |
| `wasmModule` | 自定义 WASM 初始化输入 |
| `bm25` | `{ k1, b }` 自定义 BM25 参数 |
| `searchOptions` | 默认搜索参数 |
| `initialQuery` | 初始 query |
| `autoSearch` | query 变化后自动搜索，默认 `true` |
| `suggest` | 是否自动补全，默认 `true` |
| `related` | 是否自动计算相关提示词，默认 `true` |
| `relatedLimit` | 相关提示词数量，默认 `8` |

### `UseSearchlightWorkerOptions`

继承 `UseSearchlightOptions`，额外支持：

| 字段 | 说明 |
| --- | --- |
| `worker` | 传入自定义 Worker 实例 |
| `workerUrl` | 传入 Worker URL，Vite 推荐使用 `?worker&url` |
| `wasmModuleUrl` | Worker 内初始化 WASM 时使用的自定义 WASM URL |

## 返回类型

### `SearchlightHit<TDocument>`

```ts
type SearchlightHit<TDocument> = {
  doc_id: number;
  score: number;
  document: string;
  snippet?: string | null;
  match_positions: Array<[number, number]>;
  matched_terms: string[];
  score_breakdown?: ScoreBreakdown | null;
  match_reasons?: MatchReason[] | null;
  item?: TDocument;
};
```

### `SearchlightRelatedSuggestion`

```ts
type SearchlightRelatedSuggestion = {
  term: string;
  score: number;
  doc_frequency: number;
  total_frequency: number;
  source_doc_ids: number[];
};
```

`relatedSuggestions` 是工具层候选召回，不是 LLM 生成。适合做：

- “相关搜索”
- “你可能还想搜”
- “下一步检索提示词”
- RAG/webfetch 候选内容筛选后的二次检索入口

## 常见使用 case

### 文档站搜索

```ts
const search = useSearchlight(docs, {
  getText: (doc) => `${doc.title}\n${doc.summary}\n${doc.content}`,
});
```

### 拼音和首字母

```ts
search.setQuery('bj');
search.suggest('bj');
```

### 模糊搜索

```ts
search.search('programing', { fuzzy: true, maxEditDistance: 2 });
```

### Explain 调试

```ts
search.search('worker search', { explain: true });
```

然后读取：

```ts
search.results[0].score_breakdown;
search.results[0].match_reasons;
```

### 动态数据源

```ts
await search.reindex(await fetchDocs());
```

主线程 hook 的 `reindex` 是同步返回，Worker hook 的 `reindex` 返回 Promise。

### AI/RAG 前置召回

```ts
await search.reindex(fetchedDocs);
const topDocs = await search.search(userPrompt, { limit: 8, explain: true });
const relatedTerms = await search.suggestRelated(userPrompt, 5);
```

上层应用再决定如何把 `topDocs` 组装给 LLM。

## 安全注意

- `snippet` 可能包含 `<em>`，如果文档来源不可信，请在渲染前清洗。
- 前端本地索引会把可搜索文本下发到浏览器，不适合敏感数据。
- 大量文档建议优先使用 `useSearchlightWorker` 或后端搜索。

## 本地开发

```bash
pnpm install
pnpm build:wasm
pnpm --filter @luhanxin/searchlight-react typecheck
pnpm --filter @luhanxin/searchlight-react build
```

## 发布

通常由 GitHub Actions 根据 `v*` tag 自动发布，版本号会由 tag 自动同步，不需要手动修改多个 manifest。

```bash
git tag v0.3.0
git push origin v0.3.0
```

本地手动发布前可先同步并校验版本：

```bash
pnpm release:sync v0.3.0
pnpm build
pnpm release:check v0.3.0
pnpm --filter @luhanxin/searchlight publish --access public
pnpm --filter @luhanxin/searchlight-react publish --access public
```
