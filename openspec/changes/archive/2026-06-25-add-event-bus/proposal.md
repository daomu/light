## Why

技术方案 §11/§14 规定全局事件队列（长度 16）作为 ISR/采样任务/渐暗定时器 → controller 的统一管道，§11 定义了 `Event` enum 的四个变体。在 controller（变更 6）动工之前，必须先把这个事件类型与队列单例落地，否则三个生产者（变更 3/4/6）无处投递、controller 无处消费。

## What Changes

- **填充 `event.rs`**：定义 `Event` enum，含 `MotionDetected` / `EnvDarkEntered { lux: f32 }` / `EnvBrightEntered { lux: f32 }` / `FadeTick` 四个变体（按 §11 主版本，不含 `LuxSample`）。
- **新增全局队列单例**：用 `once_cell::sync::Lazy` 包裹 `esp_idf_svc::queue::Queue<Event>`，容量 `config::EVENT_QUEUE_LEN`（16）；提供 `event::queue() -> &'static Queue<Event>` 访问器。
- **Event 必须 `Send` + `Copy`**：满足 FreeRTOS 队列定长元素要求；`f32` 字段是 `Copy`，整个 enum 无堆分配。

## Capabilities

### New Capabilities
- `event-bus`: 全局 FreeRTOS 事件队列与 `Event` enum，作为多生产者单消费者事件管道。

### Modified Capabilities
<!-- 无 -->

## Impact

- **代码**：填充 `event.rs`；可能新增 `event_queue.rs` 或并入 `event.rs`。
- **依赖**：使用变更 1 已引入的 `once_cell` 与 `esp-idf-svc`，无新增。
- **后续变更**：变更 3（采样任务）、变更 4（motion ISR）、变更 6（渐暗定时器 + controller）共用此队列。
- **API 形态**：队列元素的投递/接收 API 形态会影响变更 4 的 ISR 路径决策，本变更的 design 必须明确"FromISR 投递是否被封装"。
