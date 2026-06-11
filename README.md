# Searchlight

`searchlight` 是一个轻量级中英文全文搜索工具层库，核心由 Rust 实现，并通过 WebAssembly 提供浏览器侧能力。它适合做文档站、知识库、帮助中心、React 本地搜索，以及 AI/RAG 流程里的候选内容召回与上下文筛选前置层。

它只负责检索工具层能力：索引、搜索、排序、建议词、相关提示词、解释信息和 Worker 封装；不负责应用层策略、LLM 调用、websearch/webfetch 或最终回答生成。

## 能力总览

| 能力 | 说明 |
| --- | --- |
| 中英文混合分词 | 英文/数字 token + 中文 n-gram，适合 WASM 低依赖场景 |
| 倒排索引 | 内存内构建索引，返回稳定的数字 `doc_id` |
| BM25 排序 | 支持自定义 `k1`、`b` 参数 |
| 组合评分 | BM25、邻近度、覆盖度、短语命中、拼音命中综合排序 |
| 布尔查询 | `AND`、`OR`、`NOT`、`-term` |
| 短语查询 | `"web worker"` 或 `searchPhrase()` |
| 模糊搜索 | `term~2` 或 `fuzzy/maxEditDistance` |
| 拼音搜索 | `pinyin:beijing`、`py:bj`、`usePinyin` |
| 高亮片段 | 返回包含 `<em>` 的 `snippet` |
| 批量检索 | 一次执行多个 query |
| 搜索补全 | `suggest()`、`suggestWithPinyin()` |
| 相关提示词 | `suggestRelated(query, limit)` 从命中文档共现词中召回下一步搜索词候选 |
| Explain | `score_breakdown`、`match_reasons` 辅助调试与展示命中依据 |
| 缓存 | `enableCache` 对重复搜索做 LRU 缓存，索引变更自动失效 |
| Rust 后端 | 可直接作为服务端搜索引擎使用 |
| WASM 浏览器 | `@luhanxin/searchlight` 原始 WASM 包 |
| React Hooks | `@luhanxin/searchlight-react` 提供主线程 hook 与 Worker hook |

## 包结构

```text
searchlight/
├── src/                         # Rust 核心搜索引擎
├── pkg/                         # WASM npm 包：@luhanxin/searchlight
├── packages/react-hooks/        # React hooks 包：@luhanxin/searchlight-react
├── examples/react-web/          # React/Vite 示例
├── docs/                        # 接入与发布文档
└── scripts/build-react-wasm.sh  # WASM 构建脚本
```

## 安装与本地构建

### Rust / WASM 工具

```bash
cargo install wasm-bindgen-cli --version 0.2.123 --locked
rustup target add wasm32-unknown-unknown
```

### 仓库构建

```bash
pnpm install
pnpm build:wasm
pnpm build:react-hooks
```

或一次性构建：

```bash
pnpm build
```

### 验证

```bash
cargo test
cargo check --features wasm
pnpm --filter @luhanxin/searchlight-react typecheck
pnpm --filter searchlight-react-web build
```

## Rust 核心使用

### 基础搜索

```rust
use searchlight::{SearchEngine, SearchOptions};

let mut engine = SearchEngine::new();
engine.index_batch([
    "Rust 是一门现代系统编程语言，安全且高效",
    "React 可以结合 WASM 实现浏览器本地搜索",
    "拼音搜索支持 beijing、bj 这类输入",
]);

let results = engine.search_with_options(
    "bendi sousuo",
    &SearchOptions {
        fuzzy: true,
        max_edit_distance: 2,
        use_pinyin: true,
        highlight: true,
        limit: 10,
        ..SearchOptions::default()
    },
)?;

for item in results {
    println!("doc={} score={:.3}", item.doc_id, item.score);
}
```

### 查询语法

```rust
engine.search("Rust AND WASM")?;
engine.search("React OR Vue")?;
engine.search("search -remote")?;
engine.search("NOT deprecated")?;
engine.search("\"web worker\"")?;
engine.search("programing~2")?;
engine.search("pinyin:bendissousuo")?;
```

### 专用 API

```rust
engine.search_pinyin("biancheng")?;
engine.search_fuzzy("programing", 2)?;
engine.search_phrase("web worker")?;
let batch = engine.batch_search(["rust", "react", "worker"])?;
```

### Explain 调试

```rust
let results = engine.search_with_options(
    "react worker",
    &SearchOptions {
        explain: true,
        highlight: true,
        ..SearchOptions::default()
    },
)?;

let first = &results[0];
println!("{:?}", first.score_breakdown);
println!("{:?}", first.match_reasons);
```

### 搜索建议与相关提示词

```rust
let completions = engine.suggest("pro");
let pinyin_completions = engine.suggest_with_pinyin("bj");

// 不是 LLM 生成：只从已索引文档中找与 query 相关的共现词候选。
let related = engine.suggest_related("react search", 5)?;
for item in related {
    println!("{} {:.3} from {:?}", item.term, item.score, item.source_doc_ids);
}
```

`RelatedSuggestion` 字段：

| 字段 | 说明 |
| --- | --- |
| `term` | 推荐提示词 |
| `score` | 聚合相关性分数 |
| `doc_frequency` | 包含该词的文档数 |
| `total_frequency` | 全索引总词频 |
| `source_doc_ids` | 贡献该建议的 top 文档 ID |

## WASM 原始包使用

先构建 `pkg`：

```bash
./scripts/build-react-wasm.sh
```

在 Web/Vite/Node 侧：

```ts
import init, { SearchEngine } from '@luhanxin/searchlight';

await init();

const engine = new SearchEngine();
engine.indexBatch([
  'Rust 后端全文搜索',
  'React WASM 本地搜索',
  'Worker 异步搜索和 reindex',
]);

const results = engine.searchWithOptions('worker search', {
  fuzzy: true,
  maxEditDistance: 2,
  usePinyin: true,
  highlight: true,
  explain: true,
  limit: 10,
});

const suggestions = engine.suggestWithPinyin('bj');
const related = engine.suggestRelated('worker search', 8);
```

WASM API 使用 camelCase：

| Rust | WASM/TS |
| --- | --- |
| `index_batch` | `indexBatch` |
| `search_with_options` | `searchWithOptions` |
| `search_pinyin` | `searchPinyin` |
| `search_fuzzy` | `searchFuzzy` |
| `search_phrase` | `searchPhrase` |
| `suggest_with_pinyin` | `suggestWithPinyin` |
| `suggest_related` | `suggestRelated` |
| `clear_cache` | `clearCache` |
| `doc_count` | `docCount` |
| `get_document` | `getDocument` |

## React Hooks 使用

安装：

```bash
pnpm add @luhanxin/searchlight @luhanxin/searchlight-react
```

### 主线程 Hook：`useSearchlight`

```tsx
import { useSearchlight } from '@luhanxin/searchlight-react';

type Doc = { title: string; summary: string; content: string; route: string };

const docs: Doc[] = [
  { title: 'React Hooks 本地搜索', summary: 'WASM 搜索', content: 'useSearchlight 示例', route: '/hooks' },
  { title: 'Worker 异步搜索', summary: '避免主线程阻塞', content: 'useSearchlightWorker 示例', route: '/worker' },
];

export function SearchBox() {
  const search = useSearchlight(docs, {
    initialQuery: 'React',
    getText: (doc) => `${doc.title}\n${doc.summary}\n${doc.content}`,
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
      <input disabled={!search.ready} value={search.query} onChange={(event) => search.setQuery(event.target.value)} />

      {search.suggestions.map((item) => (
        <button key={item} onClick={() => search.setQuery(item)}>{item}</button>
      ))}

      {search.relatedSuggestions.map((item) => (
        <button key={item.term} onClick={() => search.setQuery(item.term)}>{item.term}</button>
      ))}

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

### Worker Hook：`useSearchlightWorker`

适合文档量较大、输入频繁、希望避免主线程阻塞的场景。

```tsx
import { useSearchlightWorker } from '@luhanxin/searchlight-react';
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const search = useSearchlightWorker(docs, {
  initialQuery: 'Worker',
  getText: (doc) => `${doc.title}\n${doc.summary}\n${doc.content}`,
  workerUrl,
  relatedLimit: 8,
  searchOptions: {
    fuzzy: true,
    usePinyin: true,
    highlight: true,
    enableCache: true,
  },
});

await search.reindex(nextDocs);
await search.search(search.query);
await search.suggestRelated(search.query);
await search.clear();
```

### Hooks 返回值

| 字段/方法 | `useSearchlight` | `useSearchlightWorker` | 说明 |
| --- | --- | --- | --- |
| `status` | sync | async | `idle/loading/ready/error` |
| `ready` | sync | async | 是否可搜索 |
| `query/setQuery` | sync | async | 当前输入 |
| `results` | sync | async | 搜索结果，带 `item` 原始文档 |
| `suggestions` | sync | async | 前缀/拼音补全 |
| `relatedSuggestions` | sync | async | 相关提示词候选 |
| `search()` | 返回数组 | 返回 Promise | 搜索 |
| `suggest()` | 返回数组 | 返回 Promise | 补全建议 |
| `suggestRelated()` | 返回数组 | 返回 Promise | 相关提示词 |
| `reindex()` | void | Promise | 重建索引 |
| `clear()` | void | Promise | 清空索引 |

## 文档站 / 知识库推荐用法

1. 把文档结构化保留在业务侧：`id/title/summary/content/route/tags`。
2. 用 `getText` 拼接可搜索字段。
3. 用返回的 `doc_id` 映射回原文档对象。
4. 用 `suggestions` 做输入补全。
5. 用 `relatedSuggestions` 做“你可能还想搜”或“下一步搜索提示词”。
6. 用 `snippet` 做结果摘要，用 `route` 跳转原文档。

## AI/RAG 工具层用法

`searchlight` 可以放在 LLM 上下文构建前：

```text
用户 prompt
  -> websearch/webfetch/文档库得到候选内容
  -> searchlight 建索引或 reindex
  -> search(prompt) 取 topK
  -> suggestRelated(prompt) 获取检索提示词候选
  -> 上层应用决定怎么组装上下文、是否调用 LLM
```

注意：`suggestRelated` 不是生成式能力，它只从已索引内容中召回共现词；真正的问题生成、query rewrite、上下文压缩策略应由上层应用或 LLM 负责。

## 示例应用

```bash
pnpm install
pnpm build:wasm
pnpm --filter searchlight-react-web dev
```

示例包含：

- `useSearchlight` 主线程本地搜索页面
- `useSearchlightWorker` Worker 搜索页面
- `reindex/clear` 动态索引演示
- 搜索补全与相关提示词展示
- 高亮、Explain、拼音、模糊搜索演示

## 更多文档

- [`docs/local-search.md`](docs/local-search.md)：Rust/WASM/React/后端/AI 检索接入详解
- [`docs/react-hooks.md`](docs/react-hooks.md)：React hooks 包完整 API
- [`docs/cicd.md`](docs/cicd.md)：CI/CD 与 npm 发布流程
- [`packages/react-hooks/README.md`](packages/react-hooks/README.md)：hooks 包 README
- [`examples/react-web/README.md`](examples/react-web/README.md)：React 示例说明

## 注意事项

- `snippet` 包含 `<em>` 高亮标签，文档来源不可信时请做 HTML 白名单清洗。
- 浏览器本地索引不适合包含敏感全文数据。
- 大文档库建议使用 Worker 或后端模式。
- `maxEditDistance` 建议保持 `1` 或 `2`。
- `SearchOptions` 在 Rust 使用 snake_case，WASM/TS 使用 camelCase。
