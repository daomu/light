## ADDED Requirements

### Requirement: LEDC PWM 驱动
`drivers/led_pwm.rs` SHALL 封装 ESP32-C3 LEDC 外设，使用 `config::PIN_LED_PWM`（GPIO5）、`config::PWM_FREQ_HZ`（4000Hz）、`config::PWM_RESOLUTION_BITS`（10-bit）固定配置，对外提供 `new` / `set_duty` / `set_off` / `set_full` 四个接口。

#### Scenario: 初始化后灯灭
- **WHEN** 调用 `LedPwm::new()` 成功返回
- **THEN** LED 不亮，PWM duty 为 0

#### Scenario: 设 duty 上限
- **WHEN** 调用 `set_duty(2000)`（超过 10-bit 最大值 1023）
- **THEN** 实际 duty 被截断为 1023，不报错

#### Scenario: 关灯后可重新点亮
- **WHEN** 调用 `set_off()` 后再调用 `set_duty(1023)`
- **THEN** LED 立即恢复满亮，无需重新初始化 LEDC

### Requirement: gamma 2.2 渐暗查表
`utils/fade_table.rs` SHALL 提供 240 项 `pub const` 数组 `FADE_TABLE`，每项按 `round(1023 * (1.0 - step/239.0)^2.2)` 预计算，索引 0 MUST 为 1023，索引 239 MUST 为 0，整个表 MUST 单调非递增。

#### Scenario: 边界值正确
- **WHEN** 查询 `fade_table::at(0)` 与 `fade_table::at(239)`
- **THEN** 分别返回 1023 与 0

#### Scenario: 越界保护
- **WHEN** 查询 `fade_table::at(300)` 或 `fade_table::at(usize::MAX)`
- **THEN** 返回 0，不 panic

#### Scenario: 单调性
- **WHEN** 对任意 `i` 满足 0 < i < 239
- **THEN** `fade_table::at(i) >= fade_table::at(i+1)`
