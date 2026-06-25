## 1. 状态与上下文

- [x] 1.1 在 `state.rs` 定义 `State` enum 与 `ControllerCtx`
- [x] 1.2 提供 `ControllerCtx::new(initial_env_dark, initial_armed) -> Self`

## 2. controller 逻辑

- [x] 2.1 在 `controller.rs` 实现 `handle_event(ctx, led, fade_timer, evt)`
- [x] 2.2 落地 Idle 三类事件（EnvDarkEntered / MotionDetected / EnvBrightEntered），含 armed 判定
- [x] 2.3 落地 Lighting 三类事件（FadeTick / EnvBrightEntered / MotionDetected）
- [x] 2.4 实现镜像更新：`EnvDarkEntered`/`EnvBrightEntered` 即使被业务忽略也要更新 `ctx.env_dark`
- [x] 2.5 实现 `auto_dark_trigger_armed` 的全部维护规则（D7 表）

## 3. 渐暗定时器

- [x] 3.1 用 `esp_idf_svc::timer` 创建 auto-reload 周期定时器，周期 5000ms
- [x] 3.2 回调内仅 `event::queue().send(FadeTick, 0)`，队列满丢弃
- [x] 3.3 封装 `FadeTimerHandle::start()` / `stop()`，start 前 stop 防重复
- [x] 3.4 回调内不操作 LED、不打日志、不访问 controller 状态

## 4. controller_task

- [x] 4.1 在 `tasks/controller_task.rs` 实现 `spawn(led, fade_timer, initial_ctx)`
- [x] 4.2 `thread::Builder::new().stack_size(4096)` 起 FreeRTOS 任务
- [x] 4.3 无限超时 `recv`，错误时日志后继续
- [x] 4.4 调用 `controller::handle_event`

## 5. 计数口径对齐

- [x] 5.1 实现后用日志 `[FADE] step=` 验证 step 从 0 推进到 239，共 240 个亮度点
- [x] 5.2 若计数口径与 spec 不符，回调 design.md D5 实现备忘并调整 `fade_step` 递增时机

## 6. 验证

- [x] 6.1 `cargo check` 通过
- [x] 6.2 临时 main：手动往队列投 `EnvDarkEntered`，观察 LED 满亮 + `[FADE]` 日志推进
- [x] 6.3 手动投 `EnvBrightEntered`，观察 LED 立即灭 + armed 复位
- [x] 6.4 手动投 `MotionDetected`（设 `env_dark=true`），观察开灯
- [x] 6.5 验证 Lighting 状态下 `MotionDetected` 被忽略
- [x] 6.6 验证持续暗（连续 `EnvDarkEntered`）+ disarmed 不重复开灯
- [x] 6.7 临时验证代码在提交前移除
