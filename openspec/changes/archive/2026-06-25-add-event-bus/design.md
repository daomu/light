## Context

变更 1 已建立 `event.rs` 占位与 `once_cell` 依赖。技术方案 §14.1 规定队列长度 16、单消费者（controller_task）。本变更只填类型与单例，不涉及消费逻辑。队列选型与 FromISR 能力直接决定变更 4 的实现路径，故本变更必须把 API 形态写清楚，作为变更 4 的前置依据。

## Goals / Non-Goals

**Goals:**
- 提供 `Event` enum 与全部四个变体。
- 提供全局队列单例访问器 `queue()`。
- 明确队列 API 的发送/接收/ISR 形态，供变更 4 据此决策。

**Non-Goals:**
- 不消费事件——消费是变更 6 的事。
- 不引入多队列或优先级队列——单队列够用。
- 不做事件聚合/节流——上层自行处理。
- 不引入 `LuxSample` 调试事件——§11 明确 MVP 不依赖。

## Decisions

### D1. Event enum 定义
```rust
#[derive(Copy, Clone, Debug)]
pub enum Event {
    MotionDetected,
    EnvDarkEntered { lux: f32 },
    EnvBrightEntered { lux: f32 },
    FadeTick,
}
```
- `Copy`：因 `f32` 是 `Copy`，整个 enum 可 `Copy`，队列拷贝语义最简。
- `Send`/`Sync`：自动特征（`f32` 满足），不需要也不可 `#[derive]`。`esp_idf_hal::task::queue::Queue<T>` 要求 `T: Send + Sync + Copy`，本类型自动满足。
- 不实现 `Eq`/`PartialEq`（`f32` 比较 NaN 麻烦，业务也不需要）。

### D2. 队列选型
使用 `esp_idf_hal::task::queue::Queue<Event>`，容量 `config::EVENT_QUEUE_LEN`（16）。
> **实现期修正**：原 design 写 `esp_idf_svc::queue::Queue`，但 esp-idf-svc 0.52.x 没有 `queue` 模块。FreeRTOS 队列封装位于 `esp_idf_hal::task::queue::Queue<T>`。行为契约不变。

- 构造：`Queue::<Event>::new(EVENT_QUEUE_LEN)` 返回 `Queue<Event>`。
- 单例化：`once_cell::sync::Lazy::new(|| Queue::new(EVENT_QUEUE_LEN))`，`Lazy` 在首次 `queue()` 时初始化，届时调度器已运行（boot-sequence 中调用），OK。

### D3. 访问器
```rust
static QUEUE: once_cell::sync::Lazy<Queue<Event>> =
    once_cell::sync::Lazy::new(|| Queue::new(EVENT_QUEUE_LEN));

pub fn queue() -> &'static Queue<Event> {
    &QUEUE
}
```
所有生产者与消费者通过 `event::queue()` 拿到 `&'static` 引用。

### D4. 发送/接收 API 形态
`esp_idf_hal::task::queue::Queue<T>` 提供：
- `send_back(&self, item: T, timeout: TickType_t) -> Result<(), EspError>`：任务上下文发送。
- `recv_front(&self, timeout: TickType_t) -> Option<(T, bool)>`：任务上下文接收。
- **ISR 自动检测**：`send_back` 内部用 `crate::interrupt::active()` 判断当前是否在中断上下文，若在 ISR 则自动调用 `xQueueGenericSendFromISR`。**无需单独的 `send_from_isr` 方法，单条路径即可同时服务任务上下文与 ISR 上下文。**

> **实现期修正**：原 design 预留"退化路径"（ISR + 中转任务），因 `send_back` 自动检测 ISR，此退化路径不再需要。变更 4 的 motion ISR 直接调用 `queue().send_back(MotionDetected, 0)`。

### D5. 投递超时
- 生产者在任务上下文（采样任务、渐暗定时器回调）投递时，超时设为 0（非阻塞）：队列满即丢弃，日志 warn。
- 生产者在 ISR 上下文投递时，超时必须是 0（ISR 内不可阻塞）。
- 消费者（controller_task）用无限超时 `BLOCK`（`TickType_t::MAX`）阻塞等待。

### D6. 队列容量选择依据
16 来自 §14.1。容量足够消化：
- 采样任务最坏每秒 1 条事件。
- 渐暗定时器每 5 秒 1 条。
- AM312 在人持续活动时可能高频，但 controller 消费速度足够（每次循环一条），队列 16 足够缓冲短时尖峰。

## Risks / Trade-offs

- **权衡：单消费者设计**——controller 是唯一消费者，无并发竞争。代价是若未来要多消费者需重构。
- **权衡：不实现 `PartialEq`**——损失可测试性，但 Event 含 `f32`，比较意义不大。
- **权衡：队列 API 在 esp-idf-hal 而非 esp-idf-svc**——crate 归属差异，对外行为一致。esp-idf-hal 的 `Queue<T>` 直接封装 FreeRTOS `xQueue*`，且自带 ISR 上下文检测，比 esp-idf-svc 更适合本项目。
