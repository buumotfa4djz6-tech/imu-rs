# IMU-RS

IM948 传感器解析库的 Rust 实现。

## 项目结构

```
imu-rs/
├── crates/
│   ├── imu-core/          # 纯协议解析层（无 IO 依赖）
│   ├── imu-transport/     # 传输层抽象 + 串口/BLE 实现
│   └── imu-gui/           # egui GUI 应用
└── examples/              # 示例代码
```

## 功能特性

- ✅ 完整的 IM948 协议解析（Command/Response/Reading）
- ✅ 串口通信支持
- ✅ BLE 蓝牙通信支持
- ✅ 实时数据可视化
- ✅ 参数配置界面
- ✅ 跨平台支持（Windows/Linux/macOS）

## 快速开始

```bash
# 构建 GUI 应用
cargo run --package imu-gui

# 运行示例
cargo run --example basic-usage
```

## License

MIT
