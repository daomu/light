//! 前缀日志宏（技术方案 §17）。
//!
//! 在 `log` crate 之上封装六个固定前缀，便于串口观测与正则匹配：
//! `[BOOT]` / `[INIT]` / `[LUX ]` / `[EVT ]` / `[LAMP]` / `[FADE]`。

/// 启动相关日志
#[macro_export]
macro_rules! log_boot {
    ($($t:tt)*) => { ::log::info!("[BOOT] {}", ::core::format_args!($($t)*)) };
}

/// 初始化相关日志
#[macro_export]
macro_rules! log_init {
    ($($t:tt)*) => { ::log::info!("[INIT] {}", ::core::format_args!($($t)*)) };
}

/// BH1750 采样读数日志
#[macro_export]
macro_rules! log_lux {
    ($($t:tt)*) => { ::log::info!("[LUX ] {}", ::core::format_args!($($t)*)) };
}

/// 事件相关日志
#[macro_export]
macro_rules! log_evt {
    ($($t:tt)*) => { ::log::info!("[EVT ] {}", ::core::format_args!($($t)*)) };
}

/// 开关灯动作日志
#[macro_export]
macro_rules! log_lamp {
    ($($t:tt)*) => { ::log::info!("[LAMP] {}", ::core::format_args!($($t)*)) };
}

/// 渐暗推进日志
#[macro_export]
macro_rules! log_fade {
    ($($t:tt)*) => { ::log::info!("[FADE] {}", ::core::format_args!($($t)*)) };
}
