## Why

夜灯的"自动开灯"依赖环境亮度判定。技术方案 §8 规定 BH1750 连续高分辨率模式、1s 采样、双阈值滞回（6 lux / 25 lux）+ 2 次暗确认 / 1 次亮确认。§13.2 决定自写极薄 service，不引外部 crate。本变更同时落地驱动与采样任务，因为二者紧密耦合：驱动只提供 `read_lux()`，判定逻辑放采样任务里。

## What Changes

- **新增 `drivers/bh1750_service.rs`**：基于 `esp-idf-hal` I2C（SDA=GPIO6, SCL=GPIO7），地址 `0x23`，封装 `init_continuous_mode()` 与 `read_lux() -> Result<f32, BhError>`。极薄，只发命令、读 2 字节、换算。
- **新增 `tasks/light_sensor_task.rs`**：FreeRTOS `std::thread`，每 `config::BH1750_SAMPLE_PERIOD_MS`（1s）读一次 lux，按 §15.1 双阈值滞回维护 `env_dark` 与 `dark_candidate_count`，在状态切换时向事件队列投递 `EnvDarkEntered{lux}` / `EnvBrightEntered{lux}`。
- **运行期 lux 值通过 `EnvDarkEntered`/`EnvBrightEntered` 事件的 `lux` 字段传递**，不向 controller 推送每次采样（§11 明确 LuxSample 仅用于日志）。

## Capabilities

### New Capabilities
- `light-sensor`: BH1750 I2C 极薄 service + 周期采样任务 + 双阈值滞回 + 暗环境/亮环境事件投递。

### Modified Capabilities
<!-- 无 -->

## Impact

- **代码**：新增 `drivers/bh1750_service.rs`、`tasks/light_sensor_task.rs`；声明新模块。
- **依赖**：使用变更 1 已引入的 `esp-idf-hal`，无新增。
- **后续变更**：变更 5（event-bus）定义 `EnvDarkEntered`/`EnvBrightEntered`；变更 6（controller）消费这些事件；变更 7（boot-sequence）在 spawn 采样任务前同步读 2 次建立初值。
- **硬件**：BH1750 VCC=3V3、ADDR=GND（地址 0x23）、SDA=GPIO6、SCL=GPIO7；模块电源灯必须硬件去除或遮蔽（§3.2.B）。
