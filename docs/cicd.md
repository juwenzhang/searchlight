# Searchlight CI/CD

仓库提供两条 GitHub Actions 工作流：

- `.github/workflows/ci.yml`：在 push / pull request 时运行 Rust 测试、WASM 构建、pnpm workspace 安装、React hooks 构建。
- `.github/workflows/release.yml`：在 `v*` tag 发布时，根据 tag 自动同步版本号、构建并发布 `@luhanxin/searchlight` 与 `@luhanxin/searchlight-react`。

## 版本策略

发布版本以 git tag 为准，不需要手动修改多个 `package.json` 或 `Cargo.toml`。

合法 tag 格式：

```text
vX.Y.Z
vX.Y.Z-alpha.1
X.Y.Z
```

例如：

```bash
git tag v0.3.0
git push origin v0.3.0
```

发布流水线会把 `v0.3.0` 解析成 `0.3.0`，并同步到：

- `Cargo.toml` 的 `[package].version`
- `pkg/package.json` 的 `version`
- `packages/react-hooks/package.json` 的 `version`
- `packages/react-hooks/package.json` 的 peer dependency：`@luhanxin/searchlight: ^0.3.0`

> CI 中的同步发生在发布工作区内，不会自动回写仓库。如果希望仓库源码也记录新版本，可以本地运行 `pnpm release:sync vX.Y.Z` 后提交。

## 发布脚本

根目录提供：

```bash
pnpm release:sync v0.3.0
pnpm release:check v0.3.0
```

### `release:sync`

根据 tag/version 同步本地版本文件，并立即校验。

```bash
pnpm release:sync v0.3.0
```

也可以通过环境变量传入：

```bash
RELEASE_VERSION=v0.3.0 pnpm release:sync
```

GitHub Actions 中使用 `GITHUB_REF_NAME`：

```bash
pnpm release:sync "${GITHUB_REF_NAME}"
```

### `release:check`

只校验，不修改文件：

```bash
pnpm release:check v0.3.0
```

校验内容：

- tag 是否是合法 semver。
- `Cargo.toml` 是否等于 tag 版本。
- `packages/react-hooks/package.json` 是否等于 tag 版本。
- `@luhanxin/searchlight` peer dependency 是否等于 `^版本号`。
- 如果 `pkg/package.json` 存在，也校验其版本。

非法 tag 会直接失败，例如：

```bash
pnpm release:check release-0.3
```

## WASM 包版本来源

`./scripts/build-react-wasm.sh` 不再写死版本号。

版本来源优先级：

1. `SEARCHLIGHT_VERSION` 环境变量。
2. `Cargo.toml` 的 `[package].version`。

发布流水线先运行 `pnpm release:sync "${GITHUB_REF_NAME}"`，所以 `pnpm build:wasm` 生成的 `pkg/package.json` 会自动使用 tag 版本。

## 必要 Secret

发布 npm 包前，在 GitHub 仓库配置：

- `NPM_TOKEN`：拥有发布权限的 npm automation token。

## Workspace

根目录使用 `pnpm-workspace.yaml` 管理：

- `pkg`：WASM npm 包 `@luhanxin/searchlight`。
- `packages/react-hooks`：React hooks npm 包 `@luhanxin/searchlight-react`。
- `examples/react-web`：React/Vite 示例应用。

## Release workflow 流程

`release.yml` 会执行：

1. Checkout。
2. 安装 Rust stable 与 `wasm32-unknown-unknown` target。
3. 安装 `wasm-bindgen-cli`。
4. 安装 pnpm / Node。
5. `pnpm release:sync "${GITHUB_REF_NAME}"`：从 tag 同步版本。
6. `pnpm build:wasm`：生成 `pkg`。
7. `pnpm install --frozen-lockfile`：安装 workspace 依赖。
8. `pnpm build:react-hooks`：构建 hooks 包。
9. `pnpm release:check "${GITHUB_REF_NAME}"`：发布前再次校验版本一致性。
10. 发布 `@luhanxin/searchlight`。
11. 发布 `@luhanxin/searchlight-react`。

## 本地发布前验证

```bash
cargo test --features serde
pnpm install
pnpm release:sync v0.3.0
pnpm build
pnpm release:check v0.3.0
```

如果只是验证当前仓库构建，不准备发布：

```bash
cargo test --features serde
pnpm install
pnpm build
```
