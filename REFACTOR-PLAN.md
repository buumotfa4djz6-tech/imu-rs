# IMU-GUI 模块拆分重构计划

## 目标
将 imu-gui 从单个 993 行的 main.rs 拆分为多个模块，遵循 egui/eframe 最佳实践，提高代码可维护性和可测试性。

## 当前问题
- 单个 main.rs 文件 993 行，职责混杂
- UI 渲染、业务逻辑、状态管理混在一起
- 难以定位 bug，难以编写测试
- 不符合"单一职责"原则

## 目标结构
```
crates/imu-gui/src/
├── main.rs              - 入口点，ImuApp 结构体定义，eframe::App 实现
├── types.rs             - 类型定义：ConnectionType, DeviceCommand, DeviceConfig 等
├── state.rs             - ImuApp 实现：状态管理方法
├── background.rs        - 后台任务：async 设备通信循环
├── connection.rs        - 连接管理：串口/BLE 扫描和连接
├── data.rs              - 数据处理：ImuReading 处理和缓冲区管理
├── config.rs            - 配置和校准：设备配置命令
└── ui/
    ├── mod.rs           - UI 协调：update 方法调用各个面板
    ├── device_panel.rs  - 左侧面板：设备发现和连接
    ├── config_panel.rs  - 右侧面板：配置和校准 UI
    └── visualization.rs - 中央面板：数据可视化图表
```

## 拆分详情

### 1. types.rs (~50 行)
**提取内容**：
- `ConnectionType` enum (Serial, Ble)
- `ConnectedDevice` enum (Serial, Ble)
- `DeviceCommand` enum (Connect, Disconnect, SetConfig, Calibrate, QueryStatus, StartAutoReport, StopAutoReport)
- `ConnectionParams` enum (Serial, Ble)
- `DeviceConfig` struct
- `CalibrationType` enum

**理由**：纯数据类型，无依赖，易于提取

### 2. state.rs (~80 行)
**提取内容**：
- `ImuApp` struct 定义
- `Default` trait 实现
- `new()` 构造函数（如果需要）

**理由**：状态定义与业务逻辑分离

### 3. background.rs (~110 行)
**提取内容**：
- `background_task<T: Transport>()` async 函数
- 命令处理逻辑
- 数据接收循环

**理由**：纯异步逻辑，无 UI 依赖，可独立测试

### 4. connection.rs (~140 行)
**提取内容**：
- `refresh_serial_ports()`
- `scan_ble_devices()`
- `connect()`
- `disconnect()`

**理由**：连接逻辑内聚，包含 Serial 和 BLE 的统一处理

### 5. data.rs (~30 行)
**提取内容**：
- `process_imu_reading()`

**理由**：纯数据转换，无副作用，易于测试

### 6. config.rs (~100 行)
**提取内容**：
- `start_accel_calibration()`
- `start_gyro_calibration()`
- `start_mag_calibration()`
- `apply_configuration()`
- `save_configuration()`
- `load_configuration()`
- `query_device_status()`
- `start_auto_report()`
- `stop_auto_report()`

**理由**：配置和校准逻辑内聚

### 7. ui/mod.rs (~50 行)
**提取内容**：
- `eframe::App` trait 的 `update()` 实现
- 面板协调逻辑

**理由**：UI 入口点，调用各个面板函数

### 8. ui/device_panel.rs (~80 行)
**提取内容**：
- 左侧面板渲染（设备发现）
- Serial/BLE 选择 UI
- 连接/断开按钮

**理由**：设备发现和连接 UI 独立

### 9. ui/config_panel.rs (~150 行)
**提取内容**：
- 右侧面板渲染（配置）
- 传感器设置 UI
- 校准按钮
- 设备控制按钮

**理由**：配置 UI 独立，可复用

### 10. ui/visualization.rs (~130 行)
**提取内容**：
- 中央面板渲染（数据可视化）
- 加速度/陀螺仪/磁力计图表
- 数据缓冲区显示

**理由**：可视化逻辑独立，可单独测试

## 预期收益

| 指标 | 当前 | 重构后 |
|------|------|--------|
| main.rs 行数 | 993 | ~100 |
| 最大文件行数 | 993 | ~150 |
| 文件数量 | 1 | 10 |
| 职责清晰度 | 低 | 高 |
| 可测试性 | 低 | 高 |

## 实施步骤

1. **创建 types.rs** - 提取所有类型定义
2. **创建 state.rs** - 提取 ImuApp 结构体和 Default
3. **创建 background.rs** - 提取后台任务函数
4. **创建 connection.rs** - 提取连接方法
5. **创建 data.rs** - 提取数据处理方法
6. **创建 config.rs** - 提取配置和校准方法
7. **创建 ui/ 目录和文件** - 拆分 UI 渲染
8. **更新 main.rs** - 保留入口点和 eframe::App 实现
9. **编译测试** - 确保所有功能正常

## 注意事项

- 保持所有现有功能不变
- 遵循 Rust 的模块可见性规则（pub/pub(crate)）
- 确保类型在不同模块间正确共享
- 保持 egui 的 UI 渲染模式（使用 `&mut self`）
