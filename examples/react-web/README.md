# Searchlight React Web 示例

`examples/react-web` 是基于 Vite React TS 的完整演示应用，用来验证 `@luhanxin/searchlight-react` 与 `@luhanxin/searchlight` WASM 包在浏览器中的使用方式。

当前示例覆盖：

- `useSearchlight`：主线程 WASM 本地搜索。
- `useSearchlightWorker`：Web Worker 异步搜索。
- 搜索补全：`suggest` / `suggestWithPinyin`。
- 相关提示词：`suggestRelated` / `relatedSuggestions`。
- 动态索引：`reindex`。
- 清空索引：`clear`。
- 模糊搜索：`programing` 这类拼写错误。
- 拼音/首字母搜索：`bendissousuo`、`bj`。
- 高亮片段：`snippet` 中的 `<em>`。
- Explain：`score_breakdown`、`match_reasons`。
- Worker URL 打包：`@luhanxin/searchlight-react/worker?worker&url`。

## 目录结构

```text
examples/react-web/src/
├── App.tsx                      # 页面导航：本地 hook / Worker hook
├── App.css                      # 示例样式
├── data.ts                      # 示例文档与 getText 映射
├── components/
│   ├── DocumentShelf.tsx        # 当前索引文档列表
│   ├── ResultList.tsx           # 搜索结果、Explain 展示
│   ├── SearchPanel.tsx          # query、补全建议、相关提示词
│   └── UsageNotes.tsx           # 页面内代码说明
└── pages/
    ├── LocalHooksPage.tsx       # useSearchlight 示例
    └── WorkerHooksPage.tsx      # useSearchlightWorker 示例
```

## 运行

从仓库根目录执行：

```bash
cargo install wasm-bindgen-cli --version 0.2.123 --locked
rustup target add wasm32-unknown-unknown
pnpm install
pnpm build:wasm
pnpm --filter searchlight-react-web dev
```

构建验证：

```bash
pnpm --filter searchlight-react-web build
pnpm --filter searchlight-react-web lint
```

## 示例页面

### `useSearchlight` 页面

演示主线程本地 WASM 搜索：

```tsx
const search = useSearchlight(documents, {
  initialQuery: 'React Hooks',
  getText: toSearchText,
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
```

页面展示：

- 搜索状态 `status/ready`。
- `results` 搜索结果。
- `suggestions` 补全建议。
- `relatedSuggestions` 相关提示词。
- `score_breakdown` 与 `match_reasons`。

### `useSearchlightWorker` 页面

演示 Worker 中异步搜索：

```tsx
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const search = useSearchlightWorker(documents, {
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
```

页面按钮演示：

```ts
await search.reindex(workerDocuments);
await search.search(search.query);
await search.suggestRelated(search.query);
await search.clear();
```

## 从零接入你的 React 项目

安装：

```bash
pnpm add @luhanxin/searchlight @luhanxin/searchlight-react
```

定义文档类型和索引文本：

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

主线程 hook：

```tsx
const search = useSearchlight(docs, { getText: toSearchText });
```

Worker hook：

```tsx
import workerUrl from '@luhanxin/searchlight-react/worker?worker&url';

const search = useSearchlightWorker(docs, {
  getText: toSearchText,
  workerUrl,
});
```

## 相关提示词说明

`relatedSuggestions` 来自 Rust 核心 `suggestRelated(query, limit)`，它会从当前 query 命中的 top 文档里提取共现词并排序。它不是 LLM 生成，不会凭空编写问题。

适合做：

- “相关搜索”
- “你可能还想搜”
- 文档站二次检索入口
- AI/RAG 前置召回后的提示词候选

## 安全注意

- `snippet` 使用 `<em>` 做高亮，真实业务里请确认文档来源可信或做 HTML 清洗。
- 浏览器本地索引会把可搜索文本下发到客户端，不适合敏感文档全文。
- 文档量较大时建议优先使用 Worker 页面里的接入方式。
