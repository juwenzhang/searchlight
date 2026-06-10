# Searchlight React Hooks 库

`packages/react-hooks` 基于 `pkg` 里的 `@luhanxin/searchlight` WASM 包封装，提供可复用的 React 本地搜索能力。

## 包结构

```text
packages/react-hooks/
├── package.json
├── tsconfig.json
├── README.md
└── src/
    ├── core.ts
    ├── hooks.ts
    ├── index.ts
    ├── provider.ts
    └── types.ts
```

## Workspace

根目录通过 `pnpm-workspace.yaml` 管理三个 workspace 包：

- `pkg`：`@luhanxin/searchlight`。
- `packages/react-hooks`：`@luhanxin/searchlight-react`。
- `examples/react-web`：示例应用。

`@luhanxin/searchlight-react` 发布时把 `@luhanxin/searchlight` 声明为 `peerDependencies`，本地开发用 `devDependencies: workspace:^` 链接到 `pkg`。

## 本地构建

```bash
pnpm install
pnpm build
```

单独构建 hooks 包：

```bash
pnpm build:react-hooks
```

`tsup` 会生成压缩后的 ESM/CJS 产物和 `index.d.ts` 类型声明，产物位于 `packages/react-hooks/dist`。

## 核心 API

- `initSearchlight()`：初始化 WASM，内部做 Promise 单例缓存。
- `createSearchlightEngine()`：创建原始 `SearchEngine`。
- `LocalSearchProvider<T>()`：封装索引、搜索、批量搜索、建议词和文档映射。
- `useSearchlightEngine()`：低层 Hook，只负责初始化并返回原始引擎。
- `useSearchlight<T>()`：高层 Hook，负责初始化、索引、搜索、建议词和状态管理。

## React 使用

```tsx
import { useSearchlight } from '@luhanxin/searchlight-react';

const { ready, query, setQuery, results, suggestions } = useSearchlight(docs, {
  initialQuery: '本地搜索',
  getText: (doc) => `${doc.title}\n${doc.summary}\n${doc.content}`,
  searchOptions: {
    fuzzy: true,
    maxEditDistance: 2,
    usePinyin: true,
    highlight: true,
    limit: 20,
  },
});
```

返回的 `results` 会在 `item` 字段上挂回原始文档，`doc_id` 仍保留 WASM 索引里的数字 ID。

## 发布

推荐通过 GitHub Actions `release.yml` 按 tag 自动发布：

```bash
git tag v0.1.0
git push origin v0.1.0
```

本地手动发布也应使用 pnpm workspace：

```bash
pnpm build
pnpm --filter @luhanxin/searchlight publish --access public --no-git-checks
pnpm --filter @luhanxin/searchlight-react publish --access public --no-git-checks
```
