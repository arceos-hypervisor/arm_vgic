.PHONY: test test-axvisor test-starry clean help list

TEST_SCRIPT := scripts/tests.sh
TEST_SCRIPT_URL := https://raw.githubusercontent.com/arceos-hypervisor/axci/dev/tests.sh

# 检测并下载测试脚本
$(TEST_SCRIPT):
	@echo "测试脚本不存在，正在从 axci 仓库下载..."
	@mkdir -p scripts
	@curl -fsSL $(TEST_SCRIPT_URL) -o $(TEST_SCRIPT)
	@chmod +x $(TEST_SCRIPT)
	@echo "下载完成: $(TEST_SCRIPT)"

# 运行测试 (使用共享测试框架)
# 用法: make test [TARGET]
# 示例:
#   make test                 # 运行所有测试
#   make test axvisor-qemu    # 运行 axvisor-qemu 测试
#   make test starry-x86_64   # 运行 starry-x86_64 测试
#   make test list            # 列出所有测试用例
test: $(TEST_SCRIPT)
	@if [ "$(words $(MAKECMDGOALS))" -eq 1 ]; then \
		echo "Running all tests..."; \
		./$(TEST_SCRIPT) -t all; \
	else \
		test_target=$$(echo "$(MAKECMDGOALS)" | cut -d' ' -f2-); \
		if [ "$$test_target" = "list" ]; then \
			./$(TEST_SCRIPT) -t list; \
		else \
			echo "Running test for $$test_target..."; \
			./$(TEST_SCRIPT) -t "$$test_target"; \
		fi; \
	fi

# 代码格式检查
fmt:
	@echo "Checking code format..."
	cargo fmt --check

# 运行 clippy
clippy:
	@echo "Running clippy..."
	cargo clippy --target aarch64-unknown-none-softfloat -- -D warnings

# 完整的 CI 检查（本地模拟）
ci-local: fmt clippy check
	@echo "All CI checks passed!"

# 清理
clean:
	@echo "Cleaning..."
	cargo clean
	rm -rf test-results

# 帮助
help:
	@echo "arm_vgic Makefile"
	@echo ""
	@echo "可用目标:"
	@echo "  test                    - 运行集成测试"
	@echo "  fmt                     - 检查代码格式"
	@echo "  clippy                  - 运行 clippy 检查"
	@echo "  ci-local                - 运行完整的 CI 检查（本地模拟）"
	@echo "  clean                   - 清理构建产物"
	@echo "  help                    - 显示此帮助信息"
	@echo ""
	@echo "测试目标用法:"
	@echo "  make test                           # 运行所有测试"
	@echo "  make test list                      # 列出所有测试用例"
	@echo "  make test axvisor-qemu              # 运行 axvisor-qemu 测试"
	@echo "  make test axvisor-board             # 运行 axvisor-board 测试"
	@echo "  make test starry-aarch64            # 运行 starry-aarch64 测试"
	@echo "  make test starry-x86_64             # 运行 starry-x86_64 测试"
	@echo ""
	@echo "测试脚本:"
	@echo "  - 如果 scripts/tests.sh 不存在，会自动从 axci 仓库下载"
	@echo "  - axci 仓库: https://github.com/arceos-hypervisor/axci"
