## Why

夜灯"黑暗中有人经过重新点亮"依赖 AM312 人体感应。技术方案 §9 规定 GPIO4 上升沿中断、ISR 仅投递 `MotionDetected` 事件、不做逻辑。本变更落地 GPIO 中断驱动与 ISR 投递路径，是事件总线（变更 5）与 controller（变更 6）的前置依赖之一。

## What Changes

- **新增 `drivers/motion_input.rs`**：基于 `esp-idf-hal` GPIO 中断，配置 GPIO4 为输入上升沿触发；ISR 回调内调用事件队列的 FromISR 投递接口发送 `MotionDetected`。
- **ISR 行为严格对齐 §9.2**：只记录事件 + 投递 + 立即返回；不读 I2C、不算状态、不开关灯、不打印复杂日志。

## Capabilities

### New Capabilities
- `motion-sensor`: AM312 GPIO 上升沿中断 + ISR 投递 `MotionDetected` 事件。

### Modified Capabilities
<!-- 无 -->

## Impact

- **代码**：新增 `drivers/motion_input.rs`；声明新模块。
- **依赖**：使用变更 1 已引入的 `esp-idf-hal`/`esp-idf-sys`，无新增。
- **后续变更**：变更 5（event-bus）定义 `MotionDetected` 与队列 FromISR 接口；变更 7（boot-sequence）在启动时初始化并挂载中断。
- **硬件**：AM312 VCC=3V3（**禁止 5V**）、OUT=GPIO4；§3.2.C 已强调。
