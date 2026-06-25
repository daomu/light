## Context

变更 1-5 已就位：骨架、LED 驱动 + fade_table、采样任务、motion ISR、事件队列。本变更是把它们串起来的"大脑"。技术方案 §7 的状态机是整个项目的核心语义。`auto_dark_trigger_armed` 解决"持续黑暗不重复自动开灯"的关键反例，§15.2 给出其维护规则。渐暗定时器采用 `esp_timer`（通过 `esp_idf_svc::timer::EspTimer`，用户已确认）。

## Goals / Non-Goals

**Goals:**
- 落地 §7.3 全部迁移规则，覆盖所有事件 × 状态组合。
- 实现 `auto_dark_trigger_armed` 的初值、变亮置位、自动开灯后清零的完整生命周期。
- 用 `EspTimer`（esp_timer）周期投递 `FadeTick`，由 controller 启停。
- controller_task 作为唯一消费者，阻塞 recv 事件。
- 渐暗 240 步语义对齐：tick 0..239，第 239 步熄灯。

**Non-Goals:**
- 不做 host 单测——controller 逻辑靠板上验收（变更 8）。
- 不做参数可配——所有阈值/时长来自 `config.rs`。
- 不做异常恢复复杂策略——遵循变更 1 的错误基线。
- 不实现"启动时读 2 次 BH1750 建立初值"——那是变更 7 的事；本变更只提供 `ControllerCtx::new(initial_env_dark, initial_armed)`。

## Decisions

### D1. 状态与上下文
```rust
pub enum State { Idle, Lighting }

pub struct ControllerCtx {
    pub state: State,
    pub env_dark: bool,                  // 来自采样任务的镜像
    pub auto_dark_trigger_armed: bool,
    pub fade_step: u16,                  // 0..240，当前渐暗步
}
```
`fade_step` 在进入 Lighting 时置 0，每收到一个 FadeTick +1。第 239 步处理后置 0 并停定时器。

### D2. 事件处理主入口
```rust
pub fn handle_event(
    ctx: &mut ControllerCtx,
    led: &mut LedPwm,
    fade_timer: &mut FadeTimerHandle,
    evt: Event,
)
```
这是纯逻辑函数（除调用 led/fade_timer 的副作用外）。便于后续如要做 host 单测时抽离。

### D3. Idle 状态迁移（§7.3.A）
| 事件 | 条件 | 动作 |
|---|---|---|
| `EnvDarkEntered{lux}` | `auto_dark_trigger_armed == true` | `led.set_duty(fade_table::at(0))`；`fade_timer.start()`；`ctx.state = Lighting`；`ctx.fade_step = 0`；`ctx.auto_dark_trigger_armed = false`；`log_lamp!("on reason=dark_edge lux={}", lux)` |
| `EnvDarkEntered{lux}` | `auto_dark_trigger_armed == false` | 忽略，仅 `log_evt!("env_dark_entered ignored, disarmed")` |
| `MotionDetected` | `env_dark == true` | 同上开灯流程，但 `reason=motion`；**不改变** `auto_dark_trigger_armed`（§15.3） |
| `MotionDetected` | `env_dark == false` | 忽略 |
| `EnvBrightEntered{lux}` | — | `ctx.env_dark = false`；`ctx.auto_dark_trigger_armed = true`；保持 Idle；`log_evt!("env_bright_entered lux={}", lux)` |

注意：`EnvDarkEntered` 即使在 Idle 且 disarmed 时被忽略，也 MUST 更新 `ctx.env_dark = true`（镜像同步）。

### D4. Lighting 状态迁移（§7.3.B）
| 事件 | 动作 |
|---|---|
| `FadeTick` | `ctx.fade_step += 1`；若 `fade_step >= 239`：`led.set_duty(0)`；`fade_timer.stop()`；`ctx.state = Idle`；`log_lamp!("off reason=fade_done")`。否则：`led.set_duty(fade_table::at(fade_step as usize))`；`log_fade!("step={} duty={}", fade_step, duty)` |
| `EnvBrightEntered{lux}` | `led.set_duty(0)`；`fade_timer.stop()`；`ctx.env_dark = false`；`ctx.auto_dark_trigger_armed = true`；`ctx.state = Idle`；`log_lamp!("off reason=env_bright lux={}", lux)` |
| `MotionDetected` | 完全忽略（§7.3.B 事件 3） |
| `EnvDarkEntered{lux}` | 更新 `ctx.env_dark = true` 镜像；其余忽略（已经在 Lighting） |

### D5. fade_step 边界与"第 239 步"语义
- 进入 Lighting 时 `fade_step = 0`，立即设 `led.set_duty(fade_table::at(0)) = 1023`（满亮）。
- 之后每个 `FadeTick` 到达时 `fade_step += 1`，然后用 `fade_table::at(fade_step)` 设 duty。
- 当 `fade_step == 239` 时，`fade_table::at(239) == 0`，LED 熄灭，停定时器，回 Idle。
- 即总共 240 个 FadeTick（编号 0 在进入时已"消费"，编号 1..239 由定时器投递，共 239 个定时器 tick；加上进入时的初值共 240 个亮度点）。**实现时以 spec 场景为准**：定时器投递的 tick 数应使亮度从 1023 经历 240 个亮度点降到 0。

> 实现备忘：进入 Lighting 调用 `set_duty(at(0))` 后，第一个 `FadeTick` 应使 `fade_step = 1` 并 `set_duty(at(1))`。即定时器实际投递 239 次 tick 使 step 从 0 推进到 239。这与"240 个 FadeTick"的口径需要在 tasks 验证时对齐日志计数。

### D6. 渐暗定时器（esp_timer / EspTimer）
- 类型：`esp_idf_svc::timer::EspTimer`（封装 ESP-IDF `esp_timer`，高分辨率定时器）。
- 周期：`config::FADE_TICK_PERIOD_MS`（5000ms），auto-reload（`every(period)`）。
- 回调：在 esp_timer 任务上下文执行（非 ISR），投递 `Event::FadeTick` 到 `event::queue()`，队列满丢弃。
- 启停接口：`EspTimer::every(Duration)` 启动周期回调、`EspTimer::cancel()` 停止。start 时若定时器已运行则先 cancel 再 every（避免重复 tick）。
- 创建时机：boot-sequence 中 `controller_task::create_fade_timer()` 创建并返回 `EspTimer<'static>`，controller 持有。

> **实现期修正**：原 design 写 "FreeRTOS software timer"，但 `esp_idf_svc::timer::EspTimer` 是 ESP-IDF 的 `esp_timer`（高分辨率定时器），并非 FreeRTOS `xTimerCreate`。功能等价：周期回调、回调在 esp_timer 任务上下文执行（非 ISR，可安全调用 `send_back`）。esp-idf-svc 未单独暴露 FreeRTOS software timer 封装。

### D7. `auto_dark_trigger_armed` 维护（§15.2）
| 事件 | armed 变化 |
|---|---|
| 启动且环境明亮 | `true` |
| 启动且环境已暗 | `false` |
| `EnvBrightEntered` | `true` |
| Idle 因 `EnvDarkEntered` 自动开灯 | `false` |
| Idle 因 `MotionDetected` 开灯 | 不变 |
| Lighting 因 FadeTick 结束回 Idle | 不变（仍 `false`） |
| Lighting 因 `EnvBrightEntered` 回 Idle | `true`（同 EnvBrightEntered 规则） |

### D8. controller_task 循环
```rust
pub fn spawn(led: LedPwm, fade_timer: FadeTimerHandle, initial_ctx: ControllerCtx) -> JoinHandle {
    thread::Builder::new().stack_size(4096).spawn(move || {
        let mut ctx = initial_ctx;
        let mut led = led;
        let mut fade_timer = fade_timer;
        loop {
            match event::queue().recv(portMAX_DELAY) {
                Ok(Some(evt)) => controller::handle_event(&mut ctx, &mut led, &mut fade_timer, evt),
                Ok(None) => continue,  // timeout 不应发生
                Err(e) => log_evt!("recv err: {}", e),  // 不退出
            }
        }
    })
}
```

## Risks / Trade-offs

- **风险：fade_step/tick 计数口径**——D5 已尽量对齐，但"240 个 FadeTick"与"定时器实际投递次数"之间容易差一。tasks 验证时须以日志中 `[FADE] step=` 计数从 0 到 239 为准，若发现是 0 到 238 或 1 到 239，回调 D5 的实现备忘调整。
- **风险：esp_timer 回调上下文**——回调在 esp_timer 任务上下文执行（非 ISR），可安全调用 `send_back`。回调内只投递事件（不读 I2C、不打日志），栈安全。
- **风险：`LedPwm`/`FadeTimerHandle` 的所有权**——controller_task 按值持有，无共享。但定时器回调若需访问同一个 `FadeTimerHandle` 做启停，可能需要 `Arc<Mutex<...>>`。设计上让回调只投递事件、由 controller 在事件处理中启停定时器，避免回调内操作定时器自身。
- **权衡：`handle_event` 副作用函数式**——损失纯函数性（`led`/`fade_timer` 是副作用），换取简单。若未来要做 host 单测，可再抽 trait。
- **权衡：MotionDetected 在 Lighting 完全忽略**——意味着人持续活动不会延长渐暗。§7.3.B 明确如此，符合产品定位（不"续杯"）。
