## 1. fade_table 生成

- [x] 1.1 用一次性脚本（host Rust 或 Python）按 `round(1023 * (1.0 - step/239.0)^2.2)` 计算 240 项
- [x] 1.2 把结果作为 `pub const FADE_TABLE: [u16; 240]` 写入 `utils/fade_table.rs`
- [x] 1.3 实现 `pub fn at(step: usize) -> u16`，越界返回 0
- [x] 1.4 在文件顶部注释写明 gamma、步数、生成方式，便于未来重生成

## 2. LEDC 驱动

- [x] 2.1 在 `drivers/led_pwm.rs` 用 `esp-idf-hal` 的 LEDC 封装配置 channel0 / timer0
- [x] 2.2 实现 `LedPwm::new()`，初始化后立即 `set_duty(0)`
- [x] 2.3 实现 `set_duty(u16)`，入参截断到 0..=1023
- [x] 2.4 实现 `set_off()` / `set_full()` 便利方法
- [x] 2.5 用 `thiserror` 定义 `LedError`

## 3. 验证

- [x] 3.1 `cargo check` 通过
- [x] 3.2 写一个临时 main 调用：`set_full()` → sleep 1s → `set_duty(512)` → sleep 1s → `set_off()`，烧录后目视 LED 先满亮再半亮再灭
- [x] 3.3 打印 `fade_table::at(0)`、`at(120)`、`at(239)` 确认边界值
- [x] 3.4 临时验证代码在提交前移除
