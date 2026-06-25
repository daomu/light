## Context

变更 1 已建立骨架。AM312 输出在感应到人体时拉高、保持一段后回低，本方案用上升沿触发即可避免保持期间重复上报（§9.1）。技术方案要求 ISR 极轻量，仅投递事件。`esp_idf_hal::task::queue::Queue::send_back` 自带 ISR 上下文检测（内部用 `crate::interrupt::active()` 判断），自动调用 `xQueueGenericSendFromISR`，因此 ISR 内可直接调用同一接口，无需退化路径。

## Goals / Non-Goals

**Goals:**
- 提供一个 `MotionInput::new(pin)` 初始化接口，挂载上升沿中断。
- ISR 内只做：投递 `MotionDetected` + 返回。
- 提供"挂载后即工作"的语义，无需上层周期性轮询。

**Non-Goals:**
- 不做软件去抖——上升沿触发 + AM312 自身保持时间已是天然去抖。
- 不做脉宽测量或保持时间解析——§9.3 不依赖脉宽。
- 不做事件合并/节流——队列满时直接丢弃，由上层（变更 6 controller 在 Lighting 状态忽略 MotionDetected）消化。

## Decisions

### D1. 中断配置
- 引脚：`config::PIN_AM312_OUT_NUM`（GPIO4）。
- 方向：输入，上拉（`Pull::Up`）。
- 触发：上升沿（`InterruptType::PosEdge`）。
- 挂载方式：`esp_idf_hal::gpio::PinDriver::input(pin, Pull::Up)` + `set_interrupt_type(PosEdge)` + `unsafe { subscribe(callback) }`。

### D2. ISR 投递接口——单路径
`esp_idf_hal::task::queue::Queue::send_back` 内部用 `crate::interrupt::active()` 检测当前是否在 ISR 上下文：
- 任务上下文：调用 `xQueueGenericSend`。
- ISR 上下文：自动调用 `xQueueGenericSendFromISR`。

因此 ISR 回调内直接 `event::queue().send_back(Event::MotionDetected, 0)` 即可，**无需"ISR + 中转任务"退化路径**。

> **实现期修正**：原 design 预留双路径（优先 / 退化），实测 `send_back` 自带 ISR 检测后简化为单路径。

### D3. ISR 内不做的事
- ❌ 读 I2C
- ❌ 访问 controller 状态
- ❌ 调用 `log_*!` 宏（日志在 esp-idf 上不一定 ISR 安全）
- ❌ 长 sleep / busy loop

ISR 体内仅：构造 `MotionDetected` 事件 → 投递队列 → 返回。

### D4. 队列投递失败处理
队列满时（容量 16）ISR 内直接丢弃本次事件，**不重试**。理由：人体事件丢失一次不影响夜灯场景——AM312 保持高电平期间不会再次上升沿，但人持续在感应范围内通常会反复触发；且 controller 在 Lighting 状态会忽略 MotionDetected，丢失无副作用。

### D5. 全局队列句柄
ISR 回调通过 `event::queue()` 拿到 `&'static Queue<Event>`（`once_cell::sync::Lazy` 单例）。`subscribe` 回调签名为 `FnMut() + Send + 'static`，闭包捕获的 `event` 模块路径指向全局 `Lazy` 队列，满足 `'static`。

## Risks / Trade-offs

- **风险：AM312 上电抖动**——AM312 上电后可能输出几次脉冲。本变更不软去抖，依赖 boot-sequence 在初始化后清空队列一次（`while queue.recv_front(0).is_some() {}`）。
- **权衡：丢弃式队列投递**——可能丢失人体事件，但符合 §9.3"事件源定位"，且 Lighting 状态忽略 MotionDetected，丢失无业务影响。
- **权衡：不打印 ISR 日志**——损失可观测性，换取 ISR 安全性。
