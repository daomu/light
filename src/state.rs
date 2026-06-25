//! 状态机状态与运行时上下文（技术方案 §7）。

/// 主状态（§7.1）。仅两个：待机监听 / 灯光会话。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// 灯灭监听
    Idle,
    /// 灯亮渐暗
    Lighting,
}

/// 控制器运行时上下文。
///
/// `env_dark` 是采样任务真值的镜像（通过 `EnvDarkEntered`/`EnvBrightEntered` 事件更新）。
/// `auto_dark_trigger_armed` 解决"持续黑暗不重复自动开灯"。
/// `fade_step` 当前渐暗步，0..=239。
#[derive(Debug, Clone, Copy)]
pub struct ControllerCtx {
    pub state: State,
    pub env_dark: bool,
    pub auto_dark_trigger_armed: bool,
    pub fade_step: u16,
}

impl ControllerCtx {
    /// 启动期构造。初值由 boot-sequence 通过同步读 2 次 BH1750 建立。
    pub fn new(initial_env_dark: bool, initial_armed: bool) -> Self {
        Self {
            state: State::Idle,
            env_dark: initial_env_dark,
            auto_dark_trigger_armed: initial_armed,
            fade_step: 0,
        }
    }
}
