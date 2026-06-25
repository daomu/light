## Why

夜灯的发光器件是单个 5mm 暖白 LED，由 ESP32 GPIO PWM 直驱。技术方案 §10 要求 4kHz / 10-bit LEDC + 240 步 gamma 2.2 渐暗查表，§4.2/§5.4 规定 GPIO5 串联 220Ω 限流电阻。在状态机（变更 6）动工之前，必须先把"亮度可被外部按 duty 索引控制"这一基础能力落地，否则 controller 无从驱动 LED。

## What Changes

- **新增 `drivers/led_pwm.rs`**：基于 `esp-idf-hal` LEDC 封装，初始化 GPIO5、4kHz、10-bit 分辨率；对外提供 `set_duty(u16)`、`set_off()`、`set_full()` 三个接口。
- **新增 `utils/fade_table.rs`**：预计算 240 项 `u16` duty 常量数组，按 `duty = round(1023 * (1.0 - step/239)^2.2)`，第 0 步=1023，第 239 步=0。硬编码 `const`，不运行时计算。
- **生命周期**：LED 句柄由 `main.rs` 创建后传入 controller，不暴露给多任务共享。

## Capabilities

### New Capabilities
- `led-control`: LEDC PWM 驱动封装 + 240 步 gamma 渐暗查表，供 controller 按步索引驱动亮度。

### Modified Capabilities
<!-- 无 -->

## Impact

- **代码**：新增 `drivers/led_pwm.rs`、`utils/fade_table.rs`；`drivers/mod.rs` 声明新模块。
- **依赖**：使用变更 1 已引入的 `esp-idf-hal`，无新增依赖。
- **后续变更**：变更 6（controller）调用 `led_pwm::set_duty(fade_table::at(step))`。
- **硬件**：GPIO5 必须串联 220Ω 后接 LED 阳极，LED 阴极接 GND（§5.4）。
