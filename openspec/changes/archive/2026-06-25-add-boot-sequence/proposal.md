## Why

技术方案 §18 给出 13 步启动顺序，§7.4 规定启动时需连续读 2 次 BH1750 建立初始 `env_dark` 与 `auto_dark_trigger_armed`，避免"插电时房间已暗就立即误开灯"。本变更把变更 1-6 的所有零件在 `main.rs` 里串成可烧录运行的完整固件，是 MVP 落地的最后一公里。

## What Changes

- **填充 `main.rs`**：按 §18 顺序执行——日志初始化 → 板载 LED 硬件前置提醒日志 → LED PWM 初始化（duty=0）→ I2C 初始化 → BH1750 初始化连续模式 → AM312 GPIO 中断挂载 → 创建事件队列单例 → 创建渐暗定时器 → spawn controller_task（带初始 ctx）→ 同步阻塞读 2 次 BH1750 建立初值 → spawn light_sensor_task（注入初值）→ 进入主任务空循环或 join。
- **板载 LED 处理**：用户已确认仅有不可控电源灯，软件不处理；仅在启动日志打印一行硬件前置提醒。
- **初始 env_dark 建立**：同步阻塞读 2 次 BH1750（间隔 1s），按双阈值判定初值；传入 controller 的 `ControllerCtx` 与 light_sensor_task 的 `initial_env_dark` 保持一致。

## Capabilities

### New Capabilities
- `boot-sequence`: `main.rs` 启动装配流程 + 初始 env_dark 同步建立 + 板载 LED 硬件前置提醒。

### Modified Capabilities
<!-- 无 -->

## Impact

- **代码**：填充 `main.rs`，串联所有模块。
- **依赖**：无新增。
- **后续变更**：变更 8（验收）依赖本变更完成的可烧录固件。
- **行为**：本变更完成后，固件可烧录到 ESP32-C3-Zero 端到端运行，覆盖 §19 全部时序。
