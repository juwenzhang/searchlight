# Searchlight React Web 示例

这个目录是通过 Vite 官方 React TS 模板创建的：

```bash
pnpm create vite@latest examples/react-web --template react-ts
```

随后只做了三类改动：

1. 在 `package.json` 中添加 `searchlight: file:../../pkg`。
2. 添加 `src/search.ts`、`src/data.ts`，封装本地 WASM 搜索和远程 API 搜索。
3. 替换 `src/App.tsx` 和 `src/App.css`，做成可交互搜索页面。

## 运行

从 `searchlight` 仓库根目录执行：

```bash
cargo install wasm-bindgen-cli --version 0.2.123 --locked
rustup target add wasm32-unknown-unknown
./scripts/build-react-wasm.sh
cd examples/react-web
pnpm install
pnpm dev
```

## 从零接入到你的 React 项目

```bash
pnpm create vite@latest react-web --template react-ts
cd react-web
pnpm install
pnpm add ../searchlight/pkg
```

然后参考：

- `src/search.ts`：Searchlight WASM 本地搜索 Provider
- `src/App.tsx`：React 搜索 UI
- `src/data.ts`：本地文档数据
