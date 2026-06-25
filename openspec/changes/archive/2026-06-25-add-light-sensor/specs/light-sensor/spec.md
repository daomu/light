## ADDED Requirements

### Requirement: BH1750 极薄 service
`drivers/bh1750_service.rs` SHALL 基于 `esp-idf-hal` I2C 封装 BH1750，地址固定为 `config::I2C_ADDR_BH1750`（0x23），提供 `new` / `init_continuous_mode` / `read_lux` 三个接口；`read_lux` MUST 按 `(raw_high << 8 | raw_low) / 1.2` 换算返回 lux 值。

#### Scenario: 连续模式初始化
- **WHEN** 调用 `init_continuous_mode()`
- **THEN** 通过 I2C 向地址 0x23 发送命令字 0x10

#### Scenario: 正常读数
- **WHEN** BH1750 已处于连续高分辨率模式且环境亮度稳定
- **THEN** `read_lux()` 返回的值在 datasheet 标称范围内，不报错

### Requirement: 双阈值滞回判定
采样任务 SHALL 维护 `env_dark` 布尔状态，按 `config::LUX_DARK_THRESHOLD`（6 lux）与 `config::LUX_BRIGHT_THRESHOLD`（25 lux）双阈值滞回切换：`lux <= 6` 进入暗候选，`lux >= 25` 立即判亮，区间内 MUST 保持当前状态。

#### Scenario: 由亮变暗需连续 2 次确认
- **WHEN** `env_dark == false` 且连续 2 次采样 `lux <= 6`
- **THEN** 设置 `env_dark = true` 并投递 `EnvDarkEntered`

#### Scenario: 单次遮挡不误触发
- **WHEN** `env_dark == false` 且仅 1 次采样 `lux <= 6`，下一次 `lux > 6`
- **THEN** 不投递 `EnvDarkEntered`，候选计数被重置为 0

#### Scenario: 由暗变亮立即触发
- **WHEN** `env_dark == true` 且任意 1 次采样 `lux >= 25`
- **THEN** 设置 `env_dark = false` 并投递 `EnvBrightEntered`

#### Scenario: 区间内保持状态
- **WHEN** 当前 `env_dark == true` 且采样 `lux = 15`
- **THEN** 状态不变，不发事件

### Requirement: 采样周期
采样任务 MUST 以 `config::BH1750_SAMPLE_PERIOD_MS`（1000ms）为周期循环读取 BH1750，单次失败 MUST NOT 中断循环或 panic。

#### Scenario: 周期稳定
- **WHEN** 任务正常运行
- **THEN** 相邻两次 `log_lux!` 时间戳间隔约为 1 秒

#### Scenario: 单次 I2C 失败容错
- **WHEN** 某次 `read_lux()` 返回错误
- **THEN** 记录 warn 日志后继续下一次周期采样，`env_dark` 与候选计数均不更新

### Requirement: 初始 env_dark 注入
采样任务 SHALL 接收一个 `initial_env_dark: bool` 参数作为 `env_dark` 初值，由 boot-sequence 在 spawn 时传入，避免刚启动时的误判。

#### Scenario: 启动已暗不立即触发
- **WHEN** boot-sequence 判定启动时环境已暗，传入 `initial_env_dark = true`
- **THEN** 采样任务首循环即处于"已暗"分支，不会因首次读到 `lux <= 6` 而重复发 `EnvDarkEntered`
