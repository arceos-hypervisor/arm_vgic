# arm_vgic Makefile
# 简化本地开发和测试流程

.PHONY: all check build test test-axvisor test-starry clean help

# 默认目标
all: check

# 代码检查
check:
	@echo "Running cargo check..."
	cargo check --target aarch64-unknown-none-softfloat

# 构建
build:
	@echo "Building..."
	cargo build --target aarch64-unknown-none-softfloat --release

# 测试脚本路径和下载地址
TEST_SCRIPT := scripts/tests.sh
TEST_SCRIPT_URL := https://raw.githubusercontent.com/arceos-hypervisor/axci/dev/tests.sh

# 检测并下载测试脚本
$(TEST_SCRIPT):
	@echo "测试脚本不存在，正在从 axci 仓库下载..."
	@mkdir -p scripts
	@curl -fsSL $(TEST_SCRIPT_URL) -o $(TEST_SCRIPT)
	@chmod +x $(TEST_SCRIPT)
	@echo "下载完成: $(TEST_SCRIPT)"

# 运行所有测试 (使用共享测试框架)
test: $(TEST_SCRIPT)
	@echo "Running all tests..."
	./$(TEST_SCRIPT)

# 仅运行 axvisor 测试
test-axvisor: $(TEST_SCRIPT)
	@echo "Running axvisor integration test..."
	./$(TEST_SCRIPT) --target axvisor

# 仅运行 StarryOS 测试
test-starry: $(TEST_SCRIPT)
	@echo "Running StarryOS integration test..."
	./$(TEST_SCRIPT) --target starry

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
	@echo "  check        - 运行 cargo check"
	@echo "  build        - 构建项目"
	@echo "  test         - 运行所有集成测试 (自动下载测试脚本)"
	@echo "  test-axvisor - 运行 axvisor 集成测试"
	@echo "  test-starry  - 运行 StarryOS 集成测试"
	@echo "  fmt          - 检查代码格式"
	@echo "  clippy       - 运行 clippy 检查"
	@echo "  ci-local     - 运行完整的 CI 检查（本地模拟）"
	@echo "  clean        - 清理构建产物"
	@echo "  help         - 显示此帮助信息"
	@echo ""
	@echo "测试脚本:"
	@echo "  - 如果 scripts/tests.sh 不存在，会自动从 axci 仓库下载"
	@echo "  - axci 仓库: https://github.com/arceos-hypervisor/axci"

