# 实现期发现与决策点（已全部确认）

本文件记录实现过程中遇到的、与原 design 描述存在偏差的事项。
全部偏差已经用户确认，design 文档已同步更新为与代码一致。

---

## Q1. 事件队列 API 来源与原 design 不一致 ✅ 已确认

**原 design（变更 5 D2）**：使用 `esp_idf_svc::queue::Queue<Event>`。
**实际**：esp-idf-svc 0.52.x 没有 `queue` 模块。FreeRTOS 队列封装位于 `esp_idf_hal::task::queue::Queue<T>`。
**已采取路径**：改用 `esp_idf_hal::task::queue::Queue<T>`。行为契约不变。
**design 已同步**：变更 5 design.md D2/D3/D4 已更新。

---

## Q2. ISR 投递事件的"退化路径"不再需要 ✅ 已确认

**原 design（变更 4 D2）**：预留"ISR + 10ms 中转任务"退化路径。
**实际**：`esp_idf_hal::task::queue::Queue::send_back` 内部用 `crate::interrupt::active()` 判断 ISR 上下文，自动调用 `xQueueGenericSendFromISR`。单条路径即可。
**已采取路径**：变更 4 的 motion ISR 内直接 `queue.send_back(MotionDetected, 0)`。
**design 已同步**：变更 4 design.md D2 已更新为单路径。

---

## Q3. 渐暗定时器使用 EspTimer 而非 FreeRTOS software timer ✅ 已确认

**原 design（变更 6 D6）**：用 FreeRTOS software timer。
**实际**：`esp_idf_svc::timer::EspTimer` 是 ESP-IDF 的 `esp_timer`（高分辨率定时器），功能等价。esp-idf-svc 未暴露 FreeRTOS `xTimerCreate` 封装。
**已采取路径**：使用 `EspTimer::every(Duration::from_millis(5000))` 实现周期 5s 的 `FadeTick` 投递。
**design 已同步**：变更 6 design.md D6 已更新。

---

## Q4. ESP32-C3 LEDC 仅低速通道 ✅ 已确认

**实际情况**：ESP32-C3 没有 LEDC 高速通道，`peripherals.ledc.timer0`/`channel0` 已是低速。
**已采取路径**：使用 `peripherals.ledc.timer0` + `peripherals.ledc.channel0`。

---

## Q5. `Event` 的 `Send`/`Sync` 是自动特征 ✅ 已确认

`esp_idf_hal::task::queue::Queue<T>` 要求 `T: Send + Sync + Copy`。`Event` enum 含 `f32`，自动满足。
**已采取路径**：`#[derive(Copy, Clone, Debug)]`，不显式 derive `Send`/`Sync`（它们是自动特征，`#[derive(Send, Sync)]` 会编译失败）。
**design 已同步**：变更 5 design.md D1 已更新。

---

## Q6. `main` 返回 `Result<()>` 而非 `()` ✅ 已确认

**已采取路径**：`fn main() -> Result<()>`，启动失败直接返回错误退出。`bootstrap_initial_env` 全部重试失败时保守当作明亮（`env_dark=false, armed=true`），不退出。

---

（全部偏差已确认，design 文档已与代码对齐）
