## 1. BH1750 service

- [x] 1.1 在 `drivers/bh1750_service.rs` 定义 `Bh1750` 结构与 `BhError`（thiserror）
- [x] 1.2 实现 `new(i2c)` 构造，不发命令
- [x] 1.3 实现 `init_continuous_mode()`：发命令字 0x10
- [x] 1.4 实现 `read_lux()`：读 2 字节 → `(h<<8|l)/1.2` → `f32`
- [x] 1.5 在 `init_continuous_mode` 后 sleep 200ms 的注释说明（实测调整）

## 2. 采样任务

- [x] 2.1 在 `tasks/light_sensor_task.rs` 实现 `spawn(bh, queue, initial_env_dark)` 函数
- [x] 2.2 用 `thread::Builder::new().stack_size(4096)` 起任务
- [x] 2.3 实现采样循环 + 双阈值滞回 + 2 次暗确认 / 1 次亮确认（§15.1）
- [x] 2.4 状态切换时 `queue.send(EnvDarkEntered{lux})` / `EnvBrightEntered{lux}`
- [x] 2.5 单次 read 失败：warn + 跳过 + 不更新状态
- [x] 2.6 队列满：warn + 丢事件（不阻塞采样任务）

## 3. 验证

- [x] 3.1 `cargo check` 通过
- [x] 3.2 临时 main：构造 I2C + Bh1750 + 队列，spawn 任务，遮挡/揭开传感器，串口观察 `[LUX ]` 与 `[EVT ] env_dark_entered` / `env_bright_entered`
- [x] 3.3 验证单次遮挡（< 2 秒）不会触发 `EnvDarkEntered`
- [x] 3.4 验证持续暗 2 秒后触发一次，持续暗期间不重复触发
- [x] 3.5 验证由暗变亮 1 秒内触发 `EnvBrightEntered`
- [x] 3.6 临时验证代码在提交前移除
