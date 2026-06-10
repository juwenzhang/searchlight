# Searchlight 本地搜索接入指南

本文档覆盖三类接入：

1. React Web 通过 WASM 做浏览器本地搜索。
2. Rust 后端直接引入 `searchlight`。
3. 其他服务通过 HTTP/RPC 调 Rust 搜索服务，或在 Node 侧加载 WASM 包。

## 一、WASM 打包

### 1. 安装工具

React/Vite 推荐使用稳定的 `cargo build + wasm-bindgen` 流程：

```bash
cargo install wasm-bindgen-cli --version 0.2.123 --locked
```

如果你仍要使用 `wasm-pack`，`command not found` 时可以安装：

```bash
cargo install wasm-pack --version 0.13.1 --locked
```

安装 Rust WASM target。注意 target 是 `wasm32-unknown-unknown`，不是 `wasm32-searchlight`：

```bash
rustup target add wasm32-unknown-unknown
```

### 2. React/Vite 本地搜索包

在 `searchlight` 仓库根目录执行：

```bash
./scripts/build-react-wasm.sh
```

产物位于 `pkg`，包含：

- `package.json`
- `searchlight.js`
- `searchlight_bg.wasm`
- `searchlight.d.ts`

## 二、React Web 本地搜索模式

### 1. 创建 React 项目

新项目命令：

```bash
pnpm create vite@latest react-web --template react-ts
cd react-web
pnpm install
```

本仓库已经提供完整 React 示例，并且示例目录也是通过 Vite 官方 React TS 模板创建的：

```bash
./scripts/build-react-wasm.sh
cd examples/react-web
pnpm install
pnpm dev
```

### 2. 安装本地 WASM 包

```bash
pnpm add ../searchlight/pkg
```

如果包不在 React 项目旁边，换成实际路径即可。

### 3. 定义统一类型

```ts
export type SearchResult = {
  doc_id: number;
  score: number;
  document: string;
  snippet?: string | null;
  match_positions: Array<[number, number]>;
  matched_terms: string[];
};

export type SearchOptions = {
  fuzzy?: boolean;
  maxEditDistance?: number;
  usePinyin?: boolean;
  highlight?: boolean;
  limit?: number;
};
```

### 3. 本地 WASM Provider

```ts
import init, { SearchEngine } from '@luhanxin/searchlight';
import type { SearchOptions, SearchResult } from './types';

export class LocalSearchProvider {
  private engine?: SearchEngine;

  async init(documents: string[]) {
    await init();
    this.engine = new SearchEngine();
    this.engine.indexBatch(documents);
  }

  search(query: string, options: SearchOptions = {}): SearchResult[] {
    if (!this.engine || !query.trim()) return [];
    return this.engine.searchWithOptions(query, {
      fuzzy: true,
      maxEditDistance: 2,
      usePinyin: true,
      highlight: true,
      limit: 20,
      ...options,
    }) as SearchResult[];
  }

  suggest(prefix: string): string[] {
    if (!this.engine || !prefix.trim()) return [];
    return this.engine.suggestWithPinyin(prefix) as string[];
  }
}
```

### 5. React Hook 示例

```tsx
import { useEffect, useMemo, useState } from 'react';
import { LocalSearchProvider } from './LocalSearchProvider';
import type { SearchResult } from './types';

export function useLocalSearch(documents: string[]) {
  const provider = useMemo(() => new LocalSearchProvider(), []);
  const [ready, setReady] = useState(false);
  const [results, setResults] = useState<SearchResult[]>([]);

  useEffect(() => {
    let cancelled = false;
    provider.init(documents).then(() => {
      if (!cancelled) setReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, [documents, provider]);

  return {
    ready,
    results,
    search(query: string) {
      setResults(provider.search(query));
    },
  };
}
```

### 5. 页面中使用

```tsx
const docs = [
  'Rust 是一门现代系统编程语言，安全且高效',
  'React 可以结合 WASM 实现浏览器本地搜索',
  '拼音搜索支持 beijing、bj 这类输入',
];

export function SearchBox() {
  const { ready, results, search } = useLocalSearch(docs);

  return (
    <section>
      <input
        disabled={!ready}
        placeholder="搜索文档 / 拼音 / 模糊搜索"
        onChange={(event) => search(event.target.value)}
      />
      <ul>
        {results.map((item) => (
          <li key={item.doc_id} dangerouslySetInnerHTML={{ __html: item.snippet ?? item.document }} />
        ))}
      </ul>
    </section>
  );
}
```

> `snippet` 默认使用 `<em>...</em>` 包裹命中词。渲染前请确保文档来源可信，或在业务侧做 HTML 白名单清洗。

## 三、React Web 双模式支持

双模式建议在前端只依赖统一接口：

```ts
export interface SearchProvider {
  search(query: string, options?: SearchOptions): Promise<SearchResult[]> | SearchResult[];
  suggest?(prefix: string): Promise<string[]> | string[];
}
```

### 本地模式：WASM

适合：文档规模可控、需要离线/低延迟、搜索数据不敏感。

```ts
const provider = new LocalSearchProvider();
await provider.init(documents);
```

### 远程模式：后端 Rust 服务

适合：文档量大、索引需要集中管理、权限过滤复杂。

```ts
export class RemoteSearchProvider implements SearchProvider {
  constructor(private endpoint = '/api/search') {}

  async search(query: string, options: SearchOptions = {}) {
    const res = await fetch(this.endpoint, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ query, options }),
    });
    if (!res.ok) throw new Error(`search failed: ${res.status}`);
    return (await res.json()) as SearchResult[];
  }
}
```

### 模式选择

```ts
export function createSearchProvider(mode: 'local' | 'remote') {
  return mode === 'local'
    ? new LocalSearchProvider()
    : new RemoteSearchProvider('/api/search');
}
```

推荐策略：

- 默认 `local`：文档随页面下发或可缓存到 IndexedDB。
- 回退 `remote`：WASM 初始化失败、低端设备、数据量过大。
- 混合：本地搜标题/摘要，远程搜全文和权限数据。

## 四、Rust 后端服务接入

### 1. Cargo 引入

```toml
[dependencies]
searchlight = { path = "../searchlight", features = ["serde"] }
```

### 2. 搜索服务封装

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
        SearchService {
            engine: Arc::new(RwLock::new(engine)),
        }
    }

    pub fn search(&self, query: &str, options: SearchOptions) -> Vec<SearchResult> {
        self.engine
            .read()
            .expect("search index lock poisoned")
            .search_with_options(query, &options)
    }

    pub fn rebuild(&self, documents: impl IntoIterator<Item = String>) {
        let mut next = SearchEngine::new();
        next.index_batch(documents);
        *self.engine.write().expect("search index lock poisoned") = next;
    }
}
```

### 3. HTTP API 建议

请求：

```json
{
  "query": "本地搜索",
  "options": {
    "fuzzy": true,
    "max_edit_distance": 2,
    "use_pinyin": true,
    "highlight": true,
    "limit": 20
  }
}
```

响应直接返回 `Vec<SearchResult>`。

## 五、其他服务引入

### 方案 A：通过 Rust 搜索服务暴露 HTTP/RPC

适合 Java、Go、Python、PHP 等服务。其他服务只需要调用 `/api/search`，索引构建和搜索逻辑集中在 Rust 服务内。

优点：

- 搜索逻辑单点维护。
- 可以统一权限、限流和日志。
- 不要求每个服务都支持 WASM/Rust。

### 方案 B：Node 服务加载 WASM 包

适合 Node/SSR 或边缘函数：

```bash
cd searchlight
./scripts/build-react-wasm.sh
cd ../node-service
npm install ../searchlight/pkg
```

```ts
import init, { SearchEngine } from '@luhanxin/searchlight';

await init();
const engine = new SearchEngine();
engine.indexBatch(['Rust 后端搜索', 'Node 通过 WASM 调用 searchlight']);
const results = engine.searchWithOptions('wasm', { highlight: true, limit: 10 });
```

## 六、索引数据建议

前端本地模式建议下发结构化文档后拼接可搜索字段：

```ts
type Doc = { id: string; title: string; summary: string; content: string };

const documents = docs.map((doc) => `${doc.title}\n${doc.summary}\n${doc.content}`);
```

当前 WASM API 返回 `doc_id`，业务侧可以用 `doc_id` 映射回原始文档数组：

```ts
const resultDoc = docs[result.doc_id];
```

## 七、注意事项

- 浏览器本地模式不适合包含敏感数据的全文索引。
- 大索引建议分批构建，或切换远程模式。
- 高亮结果包含 HTML，渲染前请控制文档来源或做清洗。
- `maxEditDistance` 建议不超过 `2`，大值会增加模糊搜索成本。
- 拼音搜索需要索引中文词条，使用 `usePinyin: tru