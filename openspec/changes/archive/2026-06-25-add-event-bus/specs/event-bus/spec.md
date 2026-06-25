## ADDED Requirements

### Requirement: Event 类型
`event.rs` SHALL 定义 `Event` enum，含四个变体 `MotionDetected` / `EnvDarkEntered { lux: f32 }` / `EnvBrightEntered { lux: f32 }` / `FadeTick`，且该类型 MUST 满足 `Send + Sync + Copy`（`Send`/`Sync` 为自动特征，不需显式 derive）。

#### Scenario: 四个变体齐全
- **WHEN** 检查 `event.rs` 的 enum 定义
- **THEN** 上述四个变体全部存在，且 `EnvDarkEntered`/`EnvBrightEntered` 携带 `lux: f32` 字段

#### Scenario: 可跨任务传递
- **WHEN** 在一个任务构造 `Event` 后通过队列发送到另一任务
- **THEN** 编译通过，无 `Send` bound 错误

### Requirement: 全局队列单例
`event.rs` SHALL 提供全局 `esp_idf_hal::task::queue::Queue<Event>` 单例访问器 `queue() -> &'static Queue<Event>`，容量 MUST 等于 `config::EVENT_QUEUE_LEN`（16）。

#### Scenario: 单例一致
- **WHEN** 在不同任务中调用 `event::queue()`
- **THEN** 返回的队列指针指向同一底层 FreeRTOS 队列

### Requirement: 生产者非阻塞投递
所有任务上下文的生产者 SHALL 以 0 超时投递事件，队列满时 MUST 丢弃本次事件并记录 warn 日志，且 MUST NOT 阻塞生产者任务。

#### Scenario: 队列满时生产者不阻塞
- **WHEN** 队列已满且生产者尝试投递
- **THEN** 投递立即返回（失败），生产者任务继续运行下一次循环

### Requirement: 消费者阻塞等待
消费者（controller_task）SHALL 以无限超时调用 `recv` 阻塞等待事件，无事件时 MUST 让出 CPU。

#### Scenario: 空队列时消费者不忙等
- **WHEN** 队列为空
- **THEN** controller 任务处于阻塞态，不占用 CPU
