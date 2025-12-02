.PHONY: meteora-build meteora-run meteora-run-save meteora-run-latest

meteora-build:
	cargo build --release --target wasm32-unknown-unknown -p meteora

# 通用运行：自定义 START/STOP，比如：
# make meteora-run START=100000000 STOP=+100
meteora-run:
	substreams run -e mainnet.sol.streamingfast.io:443 \
		-o json \
		meteora/substreams.yaml \
		map_my_data \
		-s $(START) -t $(STOP)

# 运行并保存到 meteora_output.json
# 用法示例：make meteora-run-save START=100000000 STOP=+100
meteora-run-save:
	substreams run -e mainnet.sol.streamingfast.io:443 \
		-o json \
		meteora/substreams.yaml \
		map_my_data \
		-s $(START) -t $(STOP) | tee meteora_output.json

# 查看最近一段区块数据（从链头向前 1000 个区块到当前）
meteora-run-latest:
	substreams run -e mainnet.sol.streamingfast.io:443 \
		-o json \
		meteora/substreams.yaml \
		map_my_data \
		-s -1000 -t 0 | tee meteora_output.json

