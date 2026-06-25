## ADDED Requirements

### Requirement: 启动装配顺序
`main.rs` SHALL 按技术方案 §18 的顺序执行：日志初始化 → 板载 LED 硬件前置提醒 → LED PWM 初始化（duty=0）→ I2C 初始化 → BH1750 连续模式初始化 → AM312 中断挂载 → 事件队列单例初始化 → 渐暗定时器创建 → 同步读 2 次 BH1750 建立初值 → spawn controller_task → spawn light_sensor_task → 主循环。

#### Scenario: 启动日志顺序
- **WHEN** 固件上电启动
- **THEN** 串口日志按 `[BOOT]` → `[INIT] board_led` → `[INIT] bh1750` → `[INIT] am312` → `[INIT] pwm` → `[INIT] bootstrap` → `[BOOT] running` 顺序出现

### Requirement: 初始 env_dark 同步建立
启动时 MUST 同步阻塞读 2 次 BH1750（间隔约 1 秒）建立初始 `env_dark`：两次 `lux` 均 `<= LUX_DARK_THRESHOLD` 才视为暗；并按 §7.4 设置 `auto_dark_trigger_armed`：启动已暗为 `false`，启动明亮为 `true`。

#### Scenario: 启动时房间已暗不立即误开灯
- **WHEN** 上电时连续 2 次采样 `lux <= 6`
- **THEN** `env_dark = true`、`armed = false`，controller 不会因启动瞬间的 `EnvDarkEntered` 自动开灯

#### Scenario: 启动时房间明亮
- **WHEN** 上电时采样 `lux > 6`
- **THEN** `env_dark = false`、`armed = true`，等待真正由亮变暗时自动开灯

### Requirement: controller 与采样任务初值一致
启动时建立的 `env_dark` 初值 MUST 同时传入 `ControllerCtx::new` 与 `light_sensor_task::spawn`，二者 MUST NOT 使用不一致的初值。

#### Scenario: 两端初值同步
- **WHEN** 启动完成后立即检查 controller 与采样任务的 `env_dark`
- **THEN** 二者相等

### Requirement: 板载 LED 硬件前置提醒
启动时 MUST 打印一行日志提醒板载 LED 为硬件前置（不可控电源灯需物理遮蔽/拆除），且 MUST NOT 写任何软件拉低板载 LED 引脚的代码。

#### Scenario: 硬件前置提醒日志
- **WHEN** 启动日志输出
- **THEN** 出现一行含 `board_led` 与 `hardware` 字样的 `[INIT]` 日志

### Requirement: 启动后清空队列
spawn controller_task 之前 MUST 把事件队列中已积压的事件（如 AM312 上电抖动产生）全部排空。

#### Scenario: 启动瞬间无伪事件
- **WHEN** controller_task 启动后第一次 `recv`
- **THEN** 不会收到启动前积压的 `MotionDetected`

### Requirement: 启动期致命错误 panic
启动期任一硬件初始化（I2C / BH1750 / PWM / GPIO 中断）失败 MUST 记录错误日志后 panic，触发设备重启以便烧录阶段快速发现硬件问题。

#### Scenario: BH1750 初始化失败
- **WHEN** `Bh1750::new` 或 `init_continuous_mode` 返回错误
- **THEN** 日志记录错误后系统 panic 重启
