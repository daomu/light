## Context

变更 1-6 已就位。本变更是最终装配。技术方案 §18 给出顺序，§7.4 给出启动初值策略。关键点是"启动时同步读 2 次建立初值"——这步与正常采样的"连续 2 次暗确认"逻辑不同：启动时无论明暗都只读 2 次来定性，不要求"连续 2 次都暗"才置暗。

## Goals / Non-Goals

**Goals:**
- 严格按 §18 顺序装配。
- 同步阻塞读 2 次 BH1750 建立初始 `env_dark`。
- 把同一初值分别传给 controller 的 `ControllerCtx` 与 light_sensor_task 的 `initial_env_dark`，保持一致。
- 打印完整启动日志（§17 启动日志清单）。
- 板载 LED 硬件前置提醒（仅日志，无软件处理）。

**Non-Goals:**
- 不做错误恢复复杂策略——启动期致命错误直接 panic 或日志后停机。
- 不做 OTA / 串口命令行 / 配置加载。
- 不处理板载可控 LED——用户已确认无。
- 不引入 NVS 持久化启动计数。

## Decisions

### D1. 启动顺序（§18 落地）
```text
1. esp_idf_svc::sys::link_patches()
2. EspLogger::initialize_default()
3. log_boot!("app start, version=...")
4. log_init!("board_led: hardware-only, verify physically")    // 板载 LED 硬件前置提醒
5. let led = LedPwm::new()?                                     // duty=0
6. let i2c = I2cDriver::new(periph, sda=GPIO6, scl=GPIO7, ...)?
7. let mut bh = Bh1750::new(i2c)?; bh.init_continuous_mode()?
8. sleep(200ms)                                                 // 等 BH1750 首次测量
9. let motion = MotionInput::new(GPIO4, event::queue())?       // 挂中断
10. event::queue()                                              // 触发 Lazy 初始化
11. let fade_timer = FadeTimerHandle::new()?
12. (ctx_env_dark, ctx_armed) = bootstrap_initial_env(&mut bh) // 同步读 2 次
13. let ctx = ControllerCtx::new(ctx_env_dark, ctx_armed)
14. spawn controller_task(led, fade_timer, ctx)
15. spawn light_sensor_task(bh, event::queue(), initial_env_dark=ctx_env_dark)
16. log_boot!("running")
17. main loop: sleep 或 join 子任务
```

### D2. 初始 env_dark 同步建立
```rust
fn bootstrap_initial_env(bh: &mut Bh1750) -> (env_dark: bool, armed: bool) {
    let lux1 = bh.read_lux()?;
    sleep(1000ms);
    let lux2 = bh.read_lux()?;
    let env_dark = lux1 <= LUX_DARK_THRESHOLD && lux2 <= LUX_DARK_THRESHOLD;
    // §7.4: 启动已暗 → armed=false；启动明亮 → armed=true
    let armed = !env_dark;
    log_init!("bootstrap: lux1={} lux2={} env_dark={} armed={}", lux1, lux2, env_dark, armed);
    (env_dark, armed)
}
```
注意：这里的"启动已暗"判定是"两次都 <= 6"（取保守的暗判定），而不是正常采样的"连续 2 次暗确认"——后者要求从亮状态切换才发 EnvDarkEntered。启动时直接定性，不通过事件。

### D3. controller 与采样任务的初值一致性
`ctx_env_dark` 同时传入：
- `ControllerCtx::new(ctx_env_dark, ctx_armed)` 给 controller
- `light_sensor_task::spawn(bh, queue, initial_env_dark=ctx_env_dark)`

两边用同一份初值。之后通过 `EnvDarkEntered`/`EnvBrightEntered` 事件保持同步。

### D4. 主任务收尾
ESP-IDF std 的 main 任务不能退出（退出会触发 restart）。选项：
- **A**：main 任务 `loop { thread::sleep(Duration::MAX) }`。
- **B**：main 任务 join controller_task 句柄（controller_task 永不退出，等效 A）。
倾向 A，简单且不依赖 join 句柄的所有权。

### D5. 启动期错误处理
- I2C / BH1750 / PWM / GPIO 中断任一初始化失败：`log_init!("fail: {}", e)` 后 `panic!`（或 `EspError::panic`）。
  - 理由：启动期硬件不可用，继续运行无意义，重启循环反而便于发现。
- `bootstrap_initial_env` 的 `read_lux` 失败：重试 3 次后仍失败则 `env_dark = false`、`armed = true`（保守的"当作明亮"）+ warn 日志。这样系统至少能跑起来，等采样任务自愈。

### D6. 板载 LED 处理（用户确认：仅不可控电源灯）
- 不写任何"拉低板载 LED 引脚"的代码。
- 启动日志打印：`[INIT] board_led: hardware-only, verify physically obscured`。
- 这条日志作为产品装配前的硬件验收锚点（变更 8 验收清单第 1 条会引用）。

### D7. 启动后清空队列
AM312 上电抖动可能已往队列里塞了事件。在 spawn controller_task **之前**，循环 `queue().recv(0)` 把队列里的事件全部排空，避免 controller 启动瞬间收到伪 MotionDetected。具体放第 10.5 步。

## Risks / Trade-offs

- **风险：BH1750 首次读取返回 0**——D1 步骤 8 的 200ms sleep 缓解；若仍异常，D5 的重试机制兜底。
- **风险：AM312 上电抖动**——D7 清空队列兜底。
- **风险：`link_patches` 与 `EspLogger` 在 main 任务栈上的开销**——sdkconfig 已设 `CONFIG_ESP_MAIN_TASK_STACK_SIZE=8192`，足够。
- **权衡：启动失败 panic**——简单粗暴，但便于烧录阶段快速发现硬件问题。运行期错误仍走变更 1 基线不 panic。
- **权衡：main 任务 sleep MAX**——无 watchdog 喂养问题（ESP-IDF main 任务默认无 watchdog），但若未来加 watchdog 需改喂狗逻辑。MVP 不考虑。
