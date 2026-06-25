## 1. main.rs 装配

- [x] 1.1 实现 §18 顺序的 1-9 步：日志 → 板载 LED 提醒 → PWM → I2C → BH1750 → AM312
- [x] 1.2 第 10-11 步：触发 `event::queue()` 单例 + 创建 `FadeTimerHandle`
- [x] 1.3 第 10.5 步：循环 `recv(0)` 排空队列
- [x] 1.4 第 12 步：实现 `bootstrap_initial_env` 同步读 2 次建立初值
- [x] 1.5 第 13-15 步：构造 `ControllerCtx` + spawn controller_task + spawn light_sensor_task
- [x] 1.6 第 16-17 步：`log_boot!("running")` + 主循环 `sleep(MAX)`

## 2. 启动日志

- [x] 2.1 打印固件版本（取 `env!("CARGO_PKG_VERSION")`）
- [x] 2.2 打印 GPIO 分配（SDA/SCL/AM312/LED）
- [x] 2.3 打印 BH1750/AM312/PWM 初始化结果
- [x] 2.4 打印 bootstrap 结果（lux1/lux2/env_dark/armed）
- [x] 2.5 板载 LED 硬件前置提醒行

## 3. 错误处理

- [x] 3.1 任一硬件初始化失败：日志后 panic
- [x] 3.2 `bootstrap_initial_env` 的 read_lux 失败：重试 3 次，仍失败则保守取 `env_dark=false`/`armed=true` + warn

## 4. 验证

- [x] 4.1 `cargo build --release` 通过
- [x] 4.2 烧录后串口看到完整启动日志链
- [x] 4.3 启动时已暗：观察不会立即开灯
- [x] 4.4 启动时明亮：观察 armed=true 后，主灯关闭能触发自动开灯
- [x] 4.5 AM312 上电抖动不会让 controller 启动瞬间收到伪事件
