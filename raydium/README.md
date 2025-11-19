# Raydium AMM Substreams 模块

该模块为 Solana 上的 Raydium AMM 协议提供 Substreams 数据处理，从原始区块链数据中提取结构化事件。

## 概述

Raydium AMM 模块处理 Solana 区块，提取和转换 Raydium 相关的交易数据为使用 Protocol Buffers 的结构化事件。这能够高效地进行数据流和分析，涵盖 Raydium AMM 活动，包括交换、存款和其他协议交互。

## 功能特性

- Raydium AMM 事件的实时提取
- 使用 Protocol Buffers 的结构化数据输出
- 高性能 WASM 编译
- 支持多种事件类型（交换、存款等）
- 高效的交易过滤和解析

## 项目结构

```
raydium/
├── Cargo.toml              # Rust 依赖和配置
├── src/
│   ├── lib.rs             # 主要处理逻辑
│   └── pb/                # 生成的 protobuf 代码
├── proto/
│   └── raydium_amm.proto  # Protobuf 模式定义
└── substreams.yaml        # Substreams 配置
```

## 设置说明

### 第 1 步：创建模块目录

```bash
mkdir raydium_amm
cd raydium_amm
cargo init --lib
```

### 第 2 步：定义数据模型 (Protobuf)

创建 `proto/raydium_amm.proto`，结构如下：

```protobuf
syntax = "proto3";
package raydium_amm;

message RaydiumAmmBlockEvents {
    repeated RaydiumAmmTransactionEvents transactions = 1;
}

message RaydiumAmmTransactionEvents {
    string signature = 1;
    repeated RaydiumAmmEvent events = 2;
}

message RaydiumAmmEvent {
    oneof event {
        SwapEvent swap = 1;
        DepositEvent deposit = 2;
        // ... 其他事件
    }
}

message SwapEvent {
    string amm = 1;
    string user = 2;
    uint64 amount_in = 3;
    uint64 amount_out = 4;
    // ... 其他字段
}
```

### 第 3 步：配置 Substreams 清单

创建 `substreams.yaml`：

```yaml
specVersion: v0.1.0
package:
  name: 'raydium_amm_events'
  version: v0.1.0

imports:
  # 引入 Solana 基础数据块定义
  sol: https://spkg.io/streamingfast/solana-common-v0.3.0.spkg

protobuf:
  files:
    - raydium_amm.proto
  importPaths:
    - ./proto

binaries:
  default:
    type: wasm/rust-v1
    file: target/wasm32-unknown-unknown/release/raydium_amm.wasm

modules:
  - name: raydium_amm_events
    kind: map
    initialBlock: 200000000 # 根据协议部署时间设置
    inputs:
      - map: sol:blocks_without_votes # 输入源：Solana 区块（不含投票交易）
    output:
      type: proto:raydium_amm.RaydiumAmmBlockEvents
```

### 第 4 步：编写 Rust 处理逻辑

更新 `src/lib.rs`：

```rust
use substreams_solana::pb::sf::solana::r#type::v1::Block;
use anyhow::Error;

// 引入生成的 protobuf 代码
pub mod pb;
use pb::raydium_amm::RaydiumAmmBlockEvents;

const RAYDIUM_PROGRAM_ID: &str = "675kPX9M..."; // 具体的 Program ID

#[substreams::handlers::map]
fn raydium_amm_events(block: Block) -> Result<RaydiumAmmBlockEvents, Error> {
    let mut events = RaydiumAmmBlockEvents::default();

    for transaction in block.transactions {
        // 1. 检查交易是否成功
        if transaction.meta.as_ref().unwrap().err.is_some() {
            continue;
        }

        // 2. 遍历并解析指令 (Instruction)
        // 提示：通常建议封装一个辅助函数来处理指令解析
        // 类似于参考代码中的 parse_transaction 函数

        // ... 解析逻辑 ...
        // if instruction.program_id == RAYDIUM_PROGRAM_ID {
        //    解析 instruction.data
        //    生成 Event 对象并 push 到 events 中
        // }
    }

    Ok(events)
}
```

### 第 5 步：配置依赖

更新 `Cargo.toml` 添加必要依赖：

```toml
[package]
name = "raydium_amm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
substreams = "0.5"
substreams-solana = "0.4"
anyhow = "1.0"
prost = "0.11"

[build-dependencies]
prost-build = "0.11"
```

## 构建流程

### 生成 Protobuf 代码

```bash
substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"
```

注意：这会在 `src/pb` 下生成 Rust 代码，记得在 `lib.rs` 中使用 `mod pb;` 导出。

### 编译 WASM

```bash
cargo build --target wasm32-unknown-unknown --release
```

## 运行和调试

### 打包模块

```bash
substreams pack ./substreams.yaml
```

### 以 GUI 模式运行（推荐用于开发）

```bash
substreams gui ./substreams.yaml -e mainnet.sol.streamingfast.io:443
```

你需要一个 StreamingFast API Key，可以从 StreamingFast 平台获取。

## 核心组件

- **程序 ID 过滤**：过滤交易以仅包含 Raydium 协议交互
- **指令解析**：从相关交易中提取和解析指令数据
- **事件生成**：从解析的指令数据创建结构化事件
- **错误处理**：跳过失败的交易并优雅地处理解析错误

## 开发技巧

1. **从单一事件类型开始**，逐步添加更复杂的解析逻辑
2. **广泛使用 GUI** 进行调试 - 它提供实时数据流可视化
3. **使用已知交易进行测试** 以验证解析逻辑是否正确
4. **优化性能** - Substreams 高效处理大量数据
5. **处理边缘情况** - 始终检查交易失败和格式错误的数据

## API 要求

- 用于主网访问的 StreamingFast API 密钥
- 已安装 Substreams CLI
- 支持 WASM 目标的 Rust 工具链

## 其他资源

- [Substreams 文档](https://docs.substreams.streamingfast.io/)
- [Solana Substreams 示例](https://github.com/streamingfast/solana-substreams)
- [Raydium 程序文档](https://docs.raydium.io/)