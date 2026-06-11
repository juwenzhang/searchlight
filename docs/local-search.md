# Searchlight 本地搜索与工具层接入指南

本文覆盖 `searchlight` 的所有主要接入场景：

1. Rust 后端直接使用。
2. Web/Vite 原始 WASM 包使用。
3. React hooks 主线程搜索。
4. React hooks Worker 搜索。
5. Provider / Worker Client 低层封装。
6. 文档站、知识库、帮助中心使用模式。
7. AI/RAG 场景中的候选内容召回与上下文筛选前置层。

`searchlight` 是纯工具层，不负责应用 UI、websearch/webfetch、LLM 调用、权限策略或最终回答生成。

## 一、核心能力

| 能力 | API / 选项 |
| --- | --- |
| 基础搜索 | `search()` / `searchWithOptions()` |
| 批量搜索 | `batchSearch()` / `batch_search()` |
| 拼音搜索 | `usePinyin` / `searchPinyin()` / `search_pinyin()` |
| 模糊搜索 | `fuzzy` / `maxEditDistance` / `searchFuzzy()` |
| 短语搜索 | `"exact phrase"` / `searchPhrase()` |
| 布尔查询 | `AND` / `OR` / `NOT` / `-term` |
| 高亮 | `highlight`，返回 `snippet` |
| Explain | `explain`，返回 `score_breakdown` / `match_reasons` |
| 自动补全 | `suggest()` / `suggestWithPinyin()` |
| 相关提示词 | `suggestRelated(query, limit)` |
| 缓存 | `enableCache` / `clearCache()` |
| 动态索引 | `index()` / `indexBatch()` / `remove()` / `clear()` / hooks `reindex()` |

## 二、构建 WASM 包

安装工具：

```bash
cargo install wasm-bindgen-cli --version 0.2.123 --locked
rustup target add wasm32-unknown-unknown
```

构建：

```bash
./scripts/build-react-wasm.sh
```

输出：

```text
pkg/
├── package.json
├── searchlight.d.ts
├── searchlight.js
└── searchlight_bg.wasm
```

仅验证 WASM 编译：

```bash
cargo build --release --target wasm32-unknown-unknown --features wasm
```

## 三、Rust 后端接入

### Cargo

```toml
[dependencies]
searchlight = { path = "../searchlight", features = ["serde"] }
```

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
        explain: true,
        limit: 10,
        ..SearchOptions::default()
    },
)?;
```

### 查询语法

```rust
engine.search("Rust AND WASM")?;
engine.search("React OR Vue")?;
engine.search("worker -remote")?;
engine.search("NOT deprecated")?;
engine.search("\"web worker\"")?;
engine.search("programing~2")?;
engine.search("pinyin:bendissousuo")?;
```

### 搜索建议

```rust
let terms = engine.suggest("pro");
let pinyin_terms = engine.suggest_with_pinyin("bj");
let related = engine.suggest_related("react search", 8)?;
```

`suggestRelated` 会先检索 query 的 top 文档，再从命中文档的共现词中计算候选提示词。它是确定性工具层能力，不是生成式能力。

### 后端服务封装

```rust
use searchlight::{SearchEngine, SearchOptions, SearchResult};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SearchService {
    engine: Arc<RwLock<SearchEngine>>,
}

impl SearchService {
    pub fn new(documents: impl IntoIterator<Item = String>) -> Self {
        let mut engine = SearchEngine::new();
        engine.index_batch(documents);
        Self { engine: Arc::new(RwLock::new(engine)) }
    }

    pub fn search(&self, query: &str, options: SearchOptions) -> Vec<SearchResult> {
        self.engine
            .read()
            .expect("search index lock poisoned")
            .search_with_options(query, &options)
            .unwrap_or_default()
    }

    pub fn related(&self, query: &str, limit: usize) -> Vec<searchlight::RelatedSuggestion> {
        self.engine
            .read()
            .expect("search index lock poisoned")
            .suggest_related(query, limit)
            .unwrap_or_default()
    }

    pub fn rebuild(&self, documents: impl IntoIterator<Item = String>) {
        let mut next = SearchEngine::new();
        next.index_batch(documents);
        *self.engine.write().expect("search index lock poisoned") = next;
    }
}
```

### HTTP API 建议

请求：

```json
{
  "query": "worker search",
  "options": {
    "fuzzy": true,
    "max_edit_distance": 2,
    "use_pinyin": true,
    "highlight": true,
    "explain": true,
    "limit": 10
  },
  "related_limit": 8
}
```

响应：

```json
{
  "results": [],
  "related_suggestions": []
}
```

## 四、原始 WASM 接入

```ts
import init, { SearchEngine } from '@luhanxin/searchlight';

await init();

const engine = new SearchEngine();
engine.indexBatch([
  'Rust 后端全文搜索',
  'React WASM 本地搜索',
  'Worker 异步搜索和动态 reindex',
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

## 五、React 主线程搜索

```tsx
import { useSearchlight } from '@luhanxin/searchlight-react';

const search = useSearchlight(docs, {
  initialQuery: 'React',
  getText: (doc) => `${doc.title}\n${doc.summary}\n${doc.content}`,
  relatedLimit: 8,
  searchOptions: {
    fuzzy: true,
    usePinyin: true,
    highlight: true,
    explain: true,
  },
});
```

可直接使用：

```ts
search.results;
search.suggestions;
search.relatedSuggestions;
search.search('worker');
search.suggest('wo');
search.suggestRelated('worker search', 8);
search.reindex(nextDocs);
search.clear();
```

## 六、React Worker 搜索

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

await search.search(search.query);
await search.suggest(search.query);
await search.suggestRelated(search.query, 8);
await search.reindex(nextDocs);
await search.clear();
```

Worker 模式适合：

- 文档量较大。
- 输入高频变化。
- 搜索结果需要 Explain。
- 需要动态 `reindex`。
- 希望避免主线程阻塞。

## 七、文档站数据建模

推荐业务侧保留结构化文档：

```ts
type Doc = {
  id: string;
  title: string;
  summary: string;
  content: string;
  route: string;
  tags: string[];
};
```

索引文本由 `getText` 决定：

```ts
function toSearchText(doc: Doc) {
  return [doc.title, doc.summary, doc.content, doc.route, doc.tags.join(' ')].join('\n');
}
```

结果映射：

```ts
const original = docs[result.doc_id];
```

React hooks 已自动把原文档挂到 `hit.item`。

## 八、AI/RAG 前置召回

`searchlight` 可以作为 LLM 上下文构建前的 lexical retriever：

```text
用户 prompt
  -> 上层 websearch/webfetch/文档库拉取候选内容
  -> searchlight reindex 候选内容
  -> search(prompt, { limit: topK, explain: true })
  -> suggestRelated(prompt, limit)
  -> 上层应用组装上下文 / 调 LLM
```

示例：

```ts
await search.reindex(fetchedDocs);
const topDocs = await search.search(userPrompt, { limit: 8, explain: true });
const relatedTerms = await search.suggestRelated(userPrompt, 5);
```

边界：

- `searchlight` 不负责 websearch/webfetch。
- `searchlight` 不负责 LLM prompt 生成。
- `searchlight` 不做 embedding/vector search。
- `suggestRelated` 只返回索引内已有词项的候选排序。

## 九、模式选择

| 模式 | 适合 | 不适合 |
| --- | --- | --- |
| Rust 后端 | 大索引、权限过滤、集中服务 | 离线浏览器搜索 |
| WASM 原始包 | 非 React、Node/Edge、手动封装 | 想快速做 React 状态管理 |
| `useSearchlight` | 小中型文档站、快速接入 | 大索引、高频输入 |
| `useSearchlightWorker` | 大文档、本地离线、高频输入 | 构建工具不支持 Worker 时需额外配置 |
| 远程 API | 数据敏感、权限复杂 | 极低延迟离线体验 |

## 十、注意事项

- `snippet` 默认含 `<em>` 高亮标签，渲染前请确认数据可信或做 HTML 清洗。
- 浏览器本地索引会把可搜索文本下发到客户端，不适合敏感数据。
- `maxEditDistance` 建议不超过 `2`。
- 大文档库建议切片后索引，或者使用 Worker/后端模式。
- Rust `SearchOptions` 是 snake_case；WASM/TS 选项是 camelCase。
