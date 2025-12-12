# Meteora Substreams 模块

针对 Solana `blocks_without_votes` 过滤 Meteora 相关的交易，输出匹配的交易列表。

## 使用说明

```bash
make meteora-protogen           # 只生成 proto 对应的 Rust 代码
make meteora-build-substreams   # 调用 Substreams CLI 跑 protogen + 编译 wasm
make meteora-build              # 直接用 cargo 编译 wasm，便于调试
```

可选：将 Substreams 发布到 [Substreams Registry](https://substreams.dev)。

```bash
substreams registry login         # 登录 substreams.dev
substreams registry publish       # 发布到 substreams.dev
```

## 模块

### `meteora`

依赖 `solana-common` 提供的 `blocks_without_votes`，按常量过滤规则（`src/constant/constant.rs` 中的命名 Program ID + `FILTER_PROGRAM_IDS`）筛选目标 program，展开内层指令，输出 `proto:meteora.Meteora`。

#### 运行时参数

过滤规则目前写死在 `FILTER_PROGRAM_IDS`（直接填 base58，或 `program:<base58>` 兼容形式）。如需调整目标 program，请修改 `src/constant/constant.rs` 后重新构建。
