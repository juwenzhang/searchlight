# Searchlight

`searchlight` 是一个轻量级中英文全文搜索库，支持 Rust 后端直接引入，也支持通过 WebAssembly 适配 React/Vite 前端做浏览器本地搜索。

## 能力

- 中英文混合分词、中文 n-gram 召回，无原生压缩依赖，适合 WASM
- 倒排索引、BM25 排序、邻近度加权
- 布尔查询：`AND`、`OR`、`-term` / `NOT`
- 短语查询：`"exact phrase"`
- 模糊搜索：`term~2` 或 API 选项
- 拼音搜索：`pinyin:beijing` / `py:bj`
- 高亮片段、批量检索、自动补全
- 双模式接入：Rust 服务端模式 + React WASM 本地模式

## 一、安装 WASM 工具

React/Vite 推荐使用稳定的 `cargo build + wasm-bindgen` 流程：

```bash
cargo install wasm-bindgen-cli --version 0.2.123 --locked
```

如果你仍要使用 `wasm-pack`，`command not found` 时可以安装：

```bash
cargo install wasm-pack --version 0.13.1 --locked
```

安装 Rust WASM target，注意 target 不是 `wasm32-searchlight`：

```bash
rustup target add wasm32-unknown-unknown
```

验证：

```bash
wasm-bindgen --version
rustup target list --installed | grep wasm32-unknown-unknown
```

## 二、打包给 React/Vite 使用

在 `searchlight` 仓库根目录执行：

```bash
./scripts/build-react-wasm.sh
```

产物会生成到：

```text
pkg/
├── package.json
├── searchlight.d.ts
├── searchlight.js
└── searchlight_bg.wasm
```

如果只想验证 raw WASM 是否可编译：

```bash
cargo build --release --target wasm32-unknown-unknown --features wasm
```

## 三、创建 React Web 项目

新项目用 Vite 官方 React TS 模板创建：

```bash
pnpm create vite@latest react-web --template react-ts
cd react-web
pnpm install
```

本仓库的示例也是通过上面的 Vite 模板创建的。运行示例：

```bash
./scripts/build-react-wasm.sh
cd examples/react-web
pnpm install
pnpm dev
```

打开 Vite 输出的地址后即可在浏览器里用 React 调 WASM 做本地搜索。

## 四、React 中使用 Searchlight WASM

先在 React 项目安装本地 WASM 包：

```bash
pnpm add pwd
```

核心调用：

```ts
import { SearchEngine } from 'searchlight';

const engine = new SearchEngine();
engine.indexBatch([
  'Rust 是一门现代系统编程语言，安全且高效',
  'React 可以结合 WASM 实现浏览器本地搜索',
  '拼音搜索支持 beijing、bj 这类输入',
]);

const results = engine.searchWithOptions('bendissousuo', {
  fuzzy: true,
  maxEditDistance: 2,
  usePinyin: true,
  highlight: true,
  limit: 10,
});
```

## 五、Rust 后端引入

```toml
[dependencies]
searchlight = { path = "../searchlight", features = ["serde"] }
```

```rust
use searchlight::{SearchEngine, SearchOptions};

let mut engine = SearchEngine::new();
engine.index_batch([
    "Rust 是一门现代系统编程语言，安全且高效",
    "React 可以结合 WASM 实现浏览器本地搜索",
    "拼音搜索支持 beijing、bj 这类输入",
]);

let results = engine.search_with_options(
    "本地搜索",
    &SearchOptions {
        fuzzy: true,
        use_pinyin: true,
        highlight: true,
        limit: 10,
        ..SearchOptions::default()
    },
);
```

更多双模式 React 接入、后端服务接入和索引更新建议见 [`docs/local-search.md`](docs/local-search.md)。
