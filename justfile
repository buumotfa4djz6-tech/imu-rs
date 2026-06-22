# IMU-RS Justfile
# 用于快速执行常用命令

# 构建整个项目（开发模式）
build:
    cargo build --workspace

# 构建发布版本
build-release:
    cargo build --release --workspace

# 运行 GUI 应用
run:
    cargo run --package imu-gui

# 运行所有测试
test:
    cargo test --workspace

# 运行测试并显示输出
test-verbose:
    cargo test --workspace -- --nocapture

# 快速检查代码（比 build 快）
check:
    cargo check --workspace

# 代码质量检查
clippy:
    cargo clippy --workspace -- -D warnings

# 自动修复 clippy 警告
clippy-fix:
    cargo clippy --workspace --fix --allow-dirty --allow-staged

# 格式化所有代码
fmt:
    cargo fmt --all

# 检查代码格式（不修改）
fmt-check:
    cargo fmt --all -- --check

# 清理构建产物
clean:
    cargo clean

# 生成文档并在浏览器中打开
doc:
    cargo doc --workspace --no-deps --open

# 运行特定 crate 的测试
test-core:
    cargo test --package imu-core

test-transport:
    cargo test --package imu-transport

# 只运行 GUI 的构建检查（不运行）
check-gui:
    cargo check --package imu-gui

# 更新依赖
update:
    cargo update

# 显示依赖树
deps:
    cargo tree

# 检查过时的依赖
outdated:
    cargo outdated

# 开发模式：检查并运行（适合开发时快速验证）
dev:
    cargo check --workspace && cargo run --package imu-gui

# 完整检查：格式 + clippy + 测试
pre-commit: fmt-check clippy test

# 帮助信息
help:
    @echo "IMU-RS 常用命令:"
    @echo ""
    @echo "  build          - 构建整个项目（开发模式）"
    @echo "  build-release  - 构建发布版本"
    @echo "  run            - 运行 GUI 应用"
    @echo "  test           - 运行所有测试"
    @echo "  check          - 快速检查代码"
    @echo "  clippy         - 代码质量检查"
    @echo "  fmt            - 格式化代码"
    @echo "  clean          - 清理构建产物"
    @echo "  doc            - 生成文档并打开"
    @echo "  dev            - 检查并运行 GUI"
    @echo "  pre-commit     - 完整检查（格式 + clippy + 测试）"
    @echo ""
    @echo "使用 'just <command>' 执行命令"
