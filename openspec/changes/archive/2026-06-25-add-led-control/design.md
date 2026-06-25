## Context

变更 1 已建立 `drivers/` 与 `utils/` 骨架与 `config.rs` 常量。本变更是第一个真实驱动落地。技术方案 §10 规定 LEDC 4kHz / 10-bit / GPIO5，§10.3 规定 gamma 2.2 渐暗曲线，§10.2 规定 240 步。LED 直驱方案（无三极管/MOSFET）已在 §3.3 确认，限流电阻 220Ω 是默认值。

## Goals / Non-Goals

**Goals:**
- 提供一个最小但完整的 LED 驱动：初始化 + 设 duty + 关灯 + 满亮。
- 提供一张正确反映 gamma 2.2 感知线性曲线的 240 项查表。
- 让 controller 后续只需 `set_duty(fade_table::at(step))` 一行就能推进渐暗。

**Non-Goals:**
- 不做 LED 软开关 / 呼吸效果 / 闪烁模式——只接受外部按 duty 控制。
- 不处理 LED 过流保护——硬件限流电阻负责。
- 不做 PWM 频率/分辨率可配——固定参数来自 `config.rs`。
- 不引入动态 fade_table 计算——硬编码 const 数组。

## Decisions

### D1. LEDC 配置
- 通道：`ledc::LedcDriver` 的 channel0（ESP32-C3 LEDC 通道 0，定时器 0）。
- 引脚：`config::PIN_LED_PWM`（GPIO5）。
- 频率：`config::PWM_FREQ_HZ`（4000Hz）。
- 分辨率：`config::PWM_RESOLUTION_BITS`（10-bit，max_duty=1023）。
- 占空比范围：`0..=1023`。

### D2. 接口设计
```rust
pub struct LedPwm { driver: LedcDriver<'static, ...> }

impl LedPwm {
    pub fn new() -> Result<Self, LedError>;        // 初始化并 set_duty(0)
    pub fn set_duty(&mut self, duty: u16) -> Result<(), LedError>;  // duty 截断到 0..=1023
    pub fn set_off(&mut self) -> Result<(), LedError>;              // 等价 set_duty(0)
    pub fn set_full(&mut self) -> Result<(), LedError>;             // 等价 set_duty(1023)
}
```
错误用 `thiserror` 定义 `LedError`，遵循变更 1 的错误处理基线（运行期 set_duty 失败仅 warn 不 panic，由调用方决定是否重试）。

### D3. fade_table 设计
- 240 项 `pub const FADE_TABLE: [u16; 240]`。
- 计算公式（离线预计算，不在固件里跑）：`duty[step] = round(1023 * (1.0 - step/239.0).powf(2.2))`，step ∈ 0..239。
- 关键点：`fade_table::at(0) == 1023`、`fade_table::at(239) == 0`、单调非递增。
- 提供 `pub fn at(step: usize) -> u16`，入参越界则返回 0（视作已熄灭）。
- 预计算在实现阶段用一次性 Python/host Rust 脚本生成后，直接写入源码，不依赖运行时 `powf`。

### D4. 渐暗 tick 数量对齐
按用户确认：**240 个 FadeTick**，编号 0..239。第 0 个 tick 设 `fade_table::at(0)=1023`（满亮），第 239 个 tick 设 `fade_table::at(239)=0`（熄灭）。controller 在收到第 239 个 tick 后停定时器、回 Idle。本变更只提供表，不涉及 tick 推进逻辑（那是变更 6）。

### D5. 关灯语义
`set_off()` 必须将 duty 置 0 并保持 PWM 输出（输出持续低电平），不拆卸 LEDC 配置。重启渐暗时直接 `set_duty(fade_table::at(0))` 即可，无需重新初始化。

## Risks / Trade-offs

- **风险：esp-idf-hal LEDC API 形态**——0.44 版本的 `LedcDriver` 构造签名需实现时确认，可能需要 `PeripheralRef` 借用模式。若签名不友好，考虑直接用 `esp-idf-sys` 的 `ledc_*` C API 封装。
- **风险：LED 直驱亮度**——§22.3 提示 5mm 暖白 LED 在 3.3V 直驱下可能偏暗。本变更不解决，留给硬件选型与电阻调整（150Ω 备选）。
- **权衡：硬编码 fade_table**——损失运行时灵活性，换取零计算开销与可读性。如未来要调 gamma 或步数，直接重生成表替换。
- **权衡：不做 LED 软开关宏**——上层如需"开关"语义，由 controller 调用 `set_off()`/`set_full()` 组合实现，driver 不掺合业务语义。
