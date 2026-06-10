# Searchlight CI/CD

仓库已提供两条 GitHub Actions 工作流：

- `.github/workflows/ci.yml`：在 push / pull request 时运行 Rust 测试、WASM 构建、pnpm workspace 安装、React hooks 构建。
- `.github/workflows/release.yml`：在 `v*` tag 发布时通过 pnpm workspace 发布 `@luhanxin/searchlight` 与 `@luhanxin/searchlight-react`。

## 必要 Secret

发布 npm 包前，在 GitHub 仓库配置：

- `NPM_TOKEN`：拥有发布权限的 npm automation token。

## Workspace

根目录使用 `pnpm-workspace.yaml` 管理：

- `pkg`：WASM npm 包 `@luhanxin/searchlight`。
- `packages/react-hooks`：React hooks npm 包 `@luhanxin/searchlight-react`。
- `examples/react-web`：React/Vite 示例应用。

## 发布流程

```bash
git tag v0.1.0
git push origin v0.1.0
```

`release.yml` 会执行：

1. 安装 Rust stable 与 `wasm32-unknown-unknown` target。
2. 安装 `wasm-bindgen-cli`。
3. 运行 `pnpm build:wasm` 生成 `pkg`。
4. 运行 `pnpm install --frozen-lockfile` 安装 workspace 依赖。
5. 运行 `pnpm build:react-hooks` 构建 hooks 包。
6. 分别发布 `@luhanxin/searchlight` 与 `@luhanxin/searchlight-react`。

## 本地验证

```bash
cargo test --features serde
pnpm install
pnpm build
```
