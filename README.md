# Substreams Solana

一个基于 Substreams 技术的 Solana 区块链数据提取和清洗模块，专门用于高效处理 Solana 生态系统中的 DeFi 数据。

## 🚀 项目简介

本项目利用 StreamingFast 的 Substreams 框架，为 Solana 区块链提供高性能的数据提取和转换服务。通过 Substreams 的并行处理能力，我们能够实时地从 Solana 区块链中提取、清洗和结构化 DeFi 协议数据。

## ✨ 核心功能

- **实时数据提取**: 从 Solana 区块链实时提取交易数据和账户状态
- **智能数据清洗**: 自动过滤、转换和规范化原始区块链数据
- **高性能处理**: 利用 Substreams 的并行处理架构实现高吞吐量数据处理
- **模块化设计**: 支持不同 DeFi 协议的独立数据提取模块

## 🔧 支持的 DEX 协议

### 即将支持
- **Raydium** - Solana 最大的去中心化交易所
- **Orca** - 专业的去中心化交易所和流动性聚合器
- **Meteora** - 新一代 DeFi 协议 (已开始开发)
- **PumpFun** - Solana 生态的 Meme币 launchpad
- **其他协议** - 持续集成更多 Solana DeFi 协议

## 🏗️ 项目结构

```
substreams-solana/
├── README.md                 # 项目文档
├── meteora_dlmm/            # Meteora DLMM 模块 (开发中)
│   ├── src/                 # Rust 源代码
│   └── proto/               # Protobuf 定义文件
└── (其他 DEX 模块将陆续添加)
```

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- Protocol Buffers compiler
- Substreams CLI

### 安装步骤
```bash
# 克隆项目
git clone https://github.com/your-org/substreams-solana.git
cd substreams-solana

# 构建项目
cargo build --release
```

### 使用示例
```bash
# 运行 Substreams
substreams run -e mainnet.sol.streamingfast.io:443 \
  substreams.yaml map_block \
  -s 100 -t +200 \
  --proto-files ./proto
```

## 📊 数据输出

本项目输出标准化的结构化数据，包括：
- 交易信息
- 流动性池状态
- 价格数据
- 交易对信息
- 历史价格数据

## 🤝 贡献指南

我们欢迎社区贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

### 开发路线图
- [ ] 完成 Meteora DLMM 模块
- [ ] 添加 Raydium 支持
- [ ] 添加 Orca 支持
- [ ] 添加 PumpFun 支持
- [ ] 性能优化
- [ ] 文档完善
- [ ] 测试覆盖

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [Substreams 官方文档](https://docs.substreams.streamingfast.io/)
- [Solana 开发者文档](https://docs.solana.com/)
- [StreamingFast](https://streamingfast.io/)

## 📞 联系我们

- 项目主页: [GitHub Repository](https://github.com/your-org/substreams-solana)
- 问题反馈: [Issues](https://github.com/your-org/substreams-solana/issues)
- 社区讨论: [Discussions](https://github.com/your-org/substreams-solana/discussions)