# @luhanxin/searchlight-react

React hooks for `@luhanxin/searchlight` WebAssembly local search.

## Install

```bash
pnpm add @luhanxin/searchlight @luhanxin/searchlight-react
```

本仓库本地开发使用根目录 pnpm workspace，`tsup` 会输出压缩后的 ESM/CJS 与类型声明：

```bash
pnpm install
pnpm build:react-hooks
```

## Usage

```tsx
import { useMemo } from 'react';
import { useSearchlight } from '@luhanxin/searchlight-react';

type Doc = { title: string; content: string };

const docs: Doc[] = [
  { title: 'Rust 后端全文搜索', content: 'Searchlight 支持 BM25、拼音、模糊搜索。' },
  { title: 'React WASM 本地搜索', content: '浏览器中直接初始化 WebAssembly 索引。' },
];

export function SearchBox() {
  const documents = useMemo(() => docs, []);
  const { ready, query, setQuery, results, suggestions, error } = useSearchlight(documents, {
    initialQuery: '本地搜索',
    getText: (doc) => `${doc.title}\n${doc.content}`,
    searchOptions: {
      fuzzy: true,
      maxEditDistance: 2,
      usePinyin: true,
      highlight: true,
      limit: 10,
    },
  });

  return (
    <section>
      <input disabled={!ready} value={query} onChange={(event) => setQuery(event.target.value)} />
      {error && <p>{error.message}</p>}
      {suggestions.map((item) => (
        <button key={item} onClick={() => setQuery(item)}>{item}</button>
      ))}
      {results.map((item) => (
        <article key={item.doc_id}>
          <h3>{item.item?.title}</h3>
          <p dangerouslySetInnerHTML={{ __html: item.snippet ?? item.document }} />
        </article>
      ))}
    </section>
  );
}
```

`snippet` 可能包含 `<em>` 高亮标签；如果文档来源不可信，渲染前请做白名单清洗。
