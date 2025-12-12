.PHONY: meteora-protogen meteora-build-substreams meteora-pack meteora-build meteora-run meteora-run-save meteora-run-latest

# 仅生成 proto 对应的 Rust 代码（不编译 wasm），适合修改 proto 后快速同步
meteora-protogen:
	substreams protogen meteora/substreams.yaml

# 使用 substreams 官方构建流程，自动跑 protogen + wasm 编译
meteora-build-substreams:
	substreams build meteora/substreams.yaml

# 打包生成 .spkg，便于发布/分发
meteora-pack:
	substreams pack meteora/substreams.yaml

# 直接使用 cargo 编译 wasm（更接近底层），适合调试 Rust 代码改动
meteora-build:
	cargo build --release --target wasm32-unknown-unknown -p meteora

# 通用运行：自定义 START/STOP，比如：
# make meteora-run START=100000000 STOP=+100
meteora-run:
	substreams run -e mainnet.sol.streamingfast.io:443 \
		-o json \
		meteora/substreams.yaml \
		meteora \
		-s $(START) -t $(STOP)

# 运行并保存到 meteora_output.json
# 用法示例：make meteora-run-save START=100000000 STOP=+100
meteora-run-save:
	substreams run -e mainnet.sol.streamingfast.io:443 \
		-o json \
		meteora/substreams.yaml \
		meteora \
		-s $(START) -t $(STOP) | tee meteora_output.json

# 查看最近一段区块数据（从链头向前 1000 个区块到当前）
meteora-run-latest:
	substreams run -e mainnet.sol.streamingfast.io:443 \
		-o json \
		meteora/substreams.yaml \
		meteora \
		-s -1000 -t 0 | tee meteora_output.json
