# BLE 扫描优化计划

## 目标

优化 BLE 设备扫描功能，移除硬编码的 IM948 过滤，让用户可以通过输入框自定义筛选条件。

## 当前问题

1. **Transport 层**: `ble.rs` 中硬编码了 IM948 Service UUID 和设备名称过滤
2. **GUI 层**: 设备面板标签显示 "BLE Devices (IM948)"，限制了通用性
3. **用户体验**: 无法扫描和连接其他 BLE 设备

## 优化方案

### 1. Transport 层优化 (`crates/imu-transport/src/ble.rs`)

**移除硬编码过滤**:
- 移除 `scan()` 方法中的 Service UUID 过滤
- 移除 `start_discovery()` 中的设备名称和 Service UUID 过滤
- 扫描所有可见的 BLE 设备，不在 transport 层进行过滤

**修改位置**:
- 第 76-82 行: 移除 Service UUID 过滤，使用 `ScanFilter::default()`
- 第 191-207 行: 移除 `has_im948_service` 和 `has_im948_name` 检查

### 2. State 层优化 (`crates/imu-gui/src/state.rs`)

**添加筛选状态**:
```rust
pub struct ImuApp {
    // ... 其他字段
    pub ble_filter: String,  // 用户输入的筛选条件
}
```

**初始化**:
```rust
ble_filter: String::new(),
```

### 3. GUI 层优化 (`crates/imu-gui/src/ui/device_panel.rs`)

**更新标签**:
- 将 "BLE Devices (IM948)" 改为 "BLE Devices"

**添加筛选输入框**:
```rust
ui.horizontal(|ui| {
    ui.label("Filter:");
    ui.text_edit_singleline(&mut self.ble_filter);
});
```

**应用筛选逻辑**:
```rust
// 在排序后应用筛选
if !self.ble_filter.is_empty() {
    let filter_lower = self.ble_filter.to_lowercase();
    devices.retain(|device| {
        let name_match = device.name.as_ref()
            .map(|n| n.to_lowercase().contains(&filter_lower))
            .unwrap_or(false);
        let addr_match = device.address.to_lowercase().contains(&filter_lower);
        name_match || addr_match
    });
}
```

## 实现步骤

1. **修改 `ble.rs`**: 移除所有硬编码过滤逻辑
2. **修改 `state.rs`**: 添加 `ble_filter` 字段
3. **修改 `device_panel.rs`**: 
   - 更新标签文本
   - 添加筛选输入框
   - 实现客户端筛选逻辑
4. **测试验证**: 确保可以扫描所有 BLE 设备并通过输入框筛选

## 预期效果

- 扫描时显示所有可见的 BLE 设备
- 用户可以通过输入框按设备名称或 MAC 地址筛选
- 筛选是实时的，随输入变化
- GUI 更加通用，不再限制于 IM948 设备

## 测试要点

- [ ] 扫描能发现所有 BLE 设备
- [ ] 筛选框可以按名称过滤
- [ ] 筛选框可以按 MAC 地址过滤
- [ ] 筛选不区分大小写
- [ ] 空筛选框显示所有设备
- [ ] 编译无警告，测试通过
