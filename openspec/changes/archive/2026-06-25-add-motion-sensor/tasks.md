## 1. 实现期 API 调研

- [x] 1.1 查 `esp_idf_svc::queue::Queue` 是否暴露 `send_from_isr` 或等价方法
- [x] 1.2 查 `esp_idf_hal::gpio::Input::subscribe` 回调签名（是否可携带用户数据）
- [x] 1.3 在 `design.md` 末尾追加一段记录最终选定的路径

## 2. 驱动实现

- [x] 2.1 在 `drivers/motion_input.rs` 定义 `MotionInput` 与 `MotionError`
- [x] 2.2 实现 `new(pin, queue)`：配置 GPIO4 输入上升沿 + 挂载中断回调
- [x] 2.3 优先路径：ISR 内直接调用队列 FromISR 接口投递 `MotionDetected`
- [x] 2.4 退化路径（若 2.3 不可行）：ISR 置 `AtomicBool`，spawn 10ms 轮询中转任务消费 flag
- [x] 2.5 队列满时丢弃，不重试不阻塞

## 3. 验证

- [x] 3.1 `cargo check` 通过
- [x] 3.2 临时 main：挂中断 + 空队，手动遮挡 AM312，串口观察 `MotionDetected` 入队
- [x] 3.3 验证持续遮挡期间不重复入队（高电平保持）
- [x] 3.4 验证队列满时 ISR 不阻塞（手动填满队列后触发中断）
- [x] 3.5 临时验证代码在提交前移除
