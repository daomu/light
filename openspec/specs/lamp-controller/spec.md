# lamp-controller Specification

## Purpose
TBD - created by archiving change add-lamp-controller. Update Purpose after archive.
## Requirements
### Requirement: Idle 状态自动开灯
当 `state == Idle` 且 `auto_dark_trigger_armed == true` 时收到 `EnvDarkEntered`，controller MUST 开灯（设 duty 为 `fade_table::at(0)`）、启动渐暗定时器、进入 `Lighting`、置 `auto_dark_trigger_armed = false`。

#### Scenario: 由亮变暗自动开灯
- **WHEN** `Idle` + `armed == true` + 收到 `EnvDarkEntered`
- **THEN** LED 满亮、渐暗定时器启动、state 转 Lighting、armed 转 false

#### Scenario: 持续黑暗不重复自动开灯
- **WHEN** `Idle` + `armed == false` + 收到 `EnvDarkEntered`
- **THEN** 不开灯、保持 Idle；但 `env_dark` 镜像 MUST 更新为 true

### Requirement: 黑暗中人体触发开灯
当 `state == Idle` 且 `env_dark == true` 时收到 `MotionDetected`，controller MUST 开灯并启动渐暗，且 MUST NOT 改变 `auto_dark_trigger_armed`。

#### Scenario: 黑暗中人经过点亮
- **WHEN** `Idle` + `env_dark == true` + 收到 `MotionDetected`
- **THEN** LED 满亮、渐暗定时器启动、state 转 Lighting；`armed` 保持原值

#### Scenario: 明亮中人经过忽略
- **WHEN** `Idle` + `env_dark == false` + 收到 `MotionDetected`
- **THEN** 不开灯、保持 Idle

### Requirement: Lighting 状态渐暗推进
当 `state == Lighting` 时收到 `FadeTick`，controller MUST 把 `fade_step` 递增并按 `fade_table::at(step)` 设 duty；当 `fade_step` 达到 239 时 MUST 关灯、停渐暗定时器、回 `Idle`。

#### Scenario: 渐暗逐步推进
- **WHEN** `Lighting` + `fade_step == 10` + 收到 `FadeTick`
- **THEN** `fade_step` 变为 11，LED duty 设为 `fade_table::at(11)`

#### Scenario: 渐暗结束熄灯
- **WHEN** `Lighting` + `fade_step == 239` + 收到 `FadeTick`（或前一步已使 step 达 239）
- **THEN** LED duty 设为 0、渐暗定时器停止、state 转 Idle

### Requirement: Lighting 状态环境变亮立即关灯
当 `state == Lighting` 时收到 `EnvBrightEntered`，controller MUST 立即关灯、停渐暗定时器、置 `env_dark = false`、置 `auto_dark_trigger_armed = true`、回 `Idle`。

#### Scenario: 主灯打开夜灯即灭
- **WHEN** `Lighting` + 收到 `EnvBrightEntered`
- **THEN** LED 立即 duty=0、渐暗定时器停止、armed=true、state=Idle

### Requirement: Lighting 状态忽略人体事件
当 `state == Lighting` 时收到 `MotionDetected`，controller MUST 完全忽略（无任何动作）。

#### Scenario: 灯亮期间人经过无副作用
- **WHEN** `Lighting` + 收到 `MotionDetected`
- **THEN** 状态、亮度、定时器、armed 均不变

### Requirement: 渐暗定时器
渐暗定时器 SHALL 为 `esp_idf_svc::timer::EspTimer`（封装 ESP-IDF `esp_timer`），周期 `config::FADE_TICK_PERIOD_MS`（5000ms），auto-reload；回调 MUST 仅投递 `FadeTick` 事件到 `event::queue()`，且 MUST NOT 直接操作 LED 或状态机。

#### Scenario: 定时器只投递事件
- **WHEN** 渐暗定时器回调触发
- **THEN** 事件队列收到一个 `FadeTick`；LED 与状态机不受回调直接影响

#### Scenario: 启停可控
- **WHEN** controller 在 Lighting 进入时调用 start，在熄灯/变亮时调用 stop
- **THEN** 定时器在 stop 后不再投递 `FadeTick`

### Requirement: controller_task 单消费者
`tasks/controller_task.rs` SHALL 作为事件队列的唯一消费者，以无限超时阻塞 `recv`，收到事件后调用 `controller::handle_event`。

#### Scenario: 空闲时让出 CPU
- **WHEN** 事件队列为空
- **THEN** controller_task 处于阻塞态，不占用 CPU

#### Scenario: 不退出
- **WHEN** `recv` 返回错误
- **THEN** 记录日志后继续循环，任务不退出

