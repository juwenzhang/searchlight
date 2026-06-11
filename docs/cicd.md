# Searchlight CI/CD

仓库提供两条 GitHub Actions 工作流：

- `.github/workflows/ci.yml`：在 push / pull request 时运行 Rust 测试、WASM feature 检查、WASM 构建、pnpm workspace 安装、React hooks 类型检查与构建。
- `.github/workflows/release.yml`：在 `v*` tag 发布时，根据 tag 自动同步版本号，依次发布 Rust crate、WASM npm 包和 React hooks npm 包。

## 发布产物

| 产物 | Registry | 名称 |
| --- | --- | --- |
| Rust crate | crates.io | `luhanxin-searchlight` |
| WASM npm 包 | npm | `@luhanxin/searchlight` |
| React hooks npm 包 | npm | `@luhanxin/searchlight-react` |

Rust crate 的 package 名是 `luhanxin-searchlight`，但 `[lib] name = "searchlight"`，所以 Rust 用户仍然这样引用：

```rust
use searchlight::{SearchEngine, SearchOptions};
```

## 必要 Secrets

在 GitHub 仓库配置：

- `CRATES_IO_TOKEN`：crates.io API token，用于 `cargo publish`。
- `NPM_TOKEN`：npm automation token，用于发布两个 npm 包。

## 版本策略

发布版本以 git tag 为准，不需要手动修改多个 `package.json`、`Cargo.toml` 或 `Cargo.lock`。

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
- `Cargo.lock` 中 `luhanxin-searchlight` package 版本
- `pkg/package.json` 的 `version`
- `packages/react-hooks/package.json` 的 `version`
- `packages/react-hooks/package.json` 的 peer dependency：`@luhanxin/searchlight: ^0.3.0`

> CI 中的同步发生在发布工作区内，不会自动回写仓库。如果希望仓库源码也记录新版本，可以本地运行 `pnpm release:sync vX.Y.Z` 后提交。

## 发布脚本

根目录提供：

```bash
pnpm release:sync v0.3.0
pnpm release:check v0.3.0
pnpm test:rust
pnpm check:wasm
pnpm package:crate
pnpm publish:crate:dry-run
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
- `Cargo.toml` package name 是否是 `luhanxin-searchlight`。
- `Cargo.toml` lib name 是否是 `searchlight`。
- `Cargo.toml` 是否等于 tag 版本。
- `Cargo.lock` root package 是否等于 tag 版本。
- `packages/react-hooks/package.json` 是否等于 tag 版本。
- `@luhanxin/searchlight` peer dependency 是否等于 `^版本号`。
- 如果 `pkg/package.json` 存在，也校验其版本。

非法 tag 会直接失败，例如：

```bash
pnpm release:check release-0.3
```

## WASM 包版本来源

`./scripts/build-react-wasm.sh` 不写死版本号。

版本来源优先级：

1. `SEARCHLIGHT_VERSION` 环境变量。
2. `Cargo.toml` 的 `[package].version`。

发布流水线先运行 `pnpm release:sync "${GITHUB_REF_NAME}"`，所以 `pnpm build:wasm` 生成的 `pkg/package.json` 会自动使用 tag 版本。

## Release workflow 流程

`release.yml` 会执行：

1. Checkout。
2. 安装 Rust stable 与 `wasm32-unknown-unknown` target。
3. 安装 `wasm-bindgen-cli`。
4. 安装 pnpm / Node。
5. `pnpm release:sync "${GITHUB_REF_NAME}"`：从 tag 同步版本。
6. `pnpm test:rust`：运行 `cargo test --features serde`。
7. `pnpm check:wasm`：运行 `cargo check --features wasm`。
8. `pnpm publish:crate:dry-run`：验证 Rust crate 可以打包发布。
9. `pnpm build:wasm`：生成 `pkg`。
10. `pnpm install --frozen-lockfile`：安装 workspace 依赖。
11. `pnpm typecheck`：检查 React hooks 类型。
12. `pnpm build:react-hooks`：构建 hooks 包。
13. `pnpm release:check "${GITHUB_REF_NAME}"`：发布前再次校验版本一致性。
14. 发布 `luhanxin-searchlight` 到 crates.io。
15. 发布 `@luhanxin/searchlight` 到 npm。
16. 发布 `@luhanxin/searchlight-react` 到 npm。

## 重复运行策略

release workflow 支持在部分发布成功后重复运行：

- 如果 `luhanxin-searchlight@版本号` 已存在于 crates.io，则跳过 Rust crate 发布。
- 如果 `@luhanxin/searchlight@版本号` 已存在于 npm，则跳过 WASM npm 包发布。
- 如果 `@luhanxin/searchlight-react@版本号` 已存在于 npm，则跳过 React hooks npm 包发布。

这样可以避免“Rust 发布成功但 npm 发布失败”后重新运行 workflow 时被已存在版本卡住。

## 本地发布前验证

```bash
cargo test --features serde
cargo check --features wasm
pnpm install
pnpm release:sync v0.3.0
pnpm package:crate
pnpm publish:crate:dry-run
pnpm build
pnpm release:check v0.3.0
```

如果只是验证当前仓库构建，不准备发布：

```bash
cargo test --features serde
cargo check --features wasm
pnpm install
pnpm build
```

## 手动发布命令

不推荐日常手动发布，但必要时可以这样执行：

```bash
pnpm release:sync v0.3.0
cargo publish --allow-dirty
pnpm build:wasm
pnpm install
pnpm build:react-hooks
pnpm --filter @luhanxin/searchlight publish --access public --no-git-checks
pnpm --filter @luhanxin/searchlight-react publish --access public --no-git-checks
```
