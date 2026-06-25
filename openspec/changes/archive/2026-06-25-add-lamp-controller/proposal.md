## Why

技术方案 §7 规定两个主状态（Idle/Lighting）+ `env_dark`/`auto_dark_trigger_armed` 内部标志，§7.3 给出完整迁移规则，§6.2 规定 controller_task + 渐暗定时器。本变更落地整个状态机、控制任务与渐暗 software timer，是产品语义的核心承载者。完成后系统即可在变更 7 装配后端到端运行。

## What Changes

- **填充 `state.rs`**：定义 `State` enum（`Idle`/`Lighting`）与运行时上下文 `ControllerCtx`（含 `env_dark: bool` 镜像、`auto_dark_trigger_armed: bool`）。
- **填充 `controller.rs`**：实现 §7.3 的状态迁移规则，提供 `handle_event(&mut ctx, &mut led, &mut fade_timer, event)` 纯函数式入口。
- **填充 `tasks/controller_task.rs`**：FreeRTOS `std::thread`，无限超时 `recv` 事件，调用 `controller::handle_event`，执行动作（开灯/关灯/启停渐暗定时器）。
- **新增渐暗定时器**：FreeRTOS software timer（`esp_idf_svc::timer`），周期 `config::FADE_TICK_PERIOD_MS`（5000ms），回调内通过 `event::queue()` 投递 `FadeTick`；由 controller 启停。
- **240 个 FadeTick 语义对齐**：tick 编号 0..239，第 239 步 duty=0 即熄灯，controller 收到第 239 个 tick 后停定时器、回 Idle。

## Capabilities

### New Capabilities
- `lamp-controller`: Idle/Lighting 状态机 + controller_task + 渐暗 software timer + `auto_dark_trigger_armed` 防重入逻辑。

### Modified Capabilities
<!-- 无 -->

## Impact

- **代码**：填充 `state.rs`/`controller.rs`/`tasks/controller_task.rs`；新增 `fade_timer` 模块（并入 `controller.rs` 或独立文件）。
- **依赖**：使用变更 1 的 `esp-idf-svc`/`anyhow`/`once_cell`、变更 2 的 `led-control`、变更 5 的 `event-bus`，无新增。
- **后续变更**：变更 7（boot-sequence）spawn controller_task、初始化定时器、注入初始 ctx。
- **行为契约**：本变更完成后，所有产品规则（§4.1-§4.4、§7.3、§15.2-§15.3）落代码。
