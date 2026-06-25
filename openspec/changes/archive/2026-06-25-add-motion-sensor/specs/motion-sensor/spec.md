## ADDED Requirements

### Requirement: AM312 GPIO 上升沿中断
`drivers/motion_input.rs` SHALL 在 `config::PIN_AM312_OUT`（GPIO4）上挂载输入上升沿中断，AM312 每次输出由低到高的边沿 MUST 触发一次事件投递。

#### Scenario: 上升沿触发一次
- **WHEN** AM312 输出从低变高
- **THEN** 事件队列收到恰好一个 `MotionDetected` 事件

#### Scenario: 高电平保持期间不重复
- **WHEN** AM312 已处于高电平且未回低
- **THEN** 不再产生新的 `MotionDetected` 事件

### Requirement: ISR 极轻量
ISR 内 MUST NOT 执行 I2C 读取、状态机计算、复杂日志、开关灯等业务逻辑；SHALL 仅构造事件、投递队列、立即返回。

#### Scenario: ISR 不阻塞
- **WHEN** 中断触发
- **THEN** ISR 在毫秒级内返回，不引起 watchdog 复位

### Requirement: 队列满时丢弃
ISR 在事件队列已满时 MUST 直接丢弃本次事件，且 MUST NOT 阻塞或重试。

#### Scenario: 队列满丢单事件
- **WHEN** 队列已满（16 条未消费）且 AM312 触发上升沿
- **THEN** 本次 `MotionDetected` 被丢弃，ISR 正常返回，系统不阻塞

### Requirement: ISR 投递路径
ISR 回调 SHALL 直接调用 `event::queue().send_back(Event::MotionDetected, 0)` 投递事件。`esp_idf_hal::task::queue::Queue::send_back` 内部自动检测 ISR 上下文并调用 `xQueueGenericSendFromISR`，故 MUST NOT 额外引入中转任务或轮询 flag。

#### Scenario: 单路径投递
- **WHEN** AM312 触发上升沿，ISR 回调执行
- **THEN** `MotionDetected` 通过 `send_back` 直接进入队列，无需中转任务
