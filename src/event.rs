//! 事件类型与全局队列（技术方案 §11 / §14）。
//!
//! 全局单例队列基于 `esp_idf_hal::task::queue::Queue<T>`，容量 16。
//! `send_back` 自动检测 ISR 上下文并调用 `xQueueGenericSendFromISR`，
//! 因此 AM312 ISR 可直接调用同一接口，无需退化路径。

use esp_idf_hal::task::queue::Queue;

use crate::config::EVENT_QUEUE_LEN;

/// 系统事件（技术方案 §11）。
///
/// 不派生 `PartialEq`：`EnvDarkEntered`/`EnvBrightEntered` 携带 `f32`，
/// 浮点比较 NaN 行为不稳定，业务也不需要事件相等判定。
///
/// `Send`/`Sync` 是自动特征（f32 字段均满足），无需显式 derive；
/// `esp_idf_hal::task::queue::Queue<T>` 要求 `T: Send + Sync + Copy`，本类型满足。
#[derive(Copy, Clone, Debug)]
pub enum Event {
    /// AM312 上升沿：检测到人经过
    MotionDetected,
    /// 环境由亮变暗（经 2 次确认），携带触发时的 lux
    EnvDarkEntered { lux: f32 },
    /// 环境由暗变亮（1 次即触发），携带触发时的 lux
    EnvBrightEntered { lux: f32 },
    /// 渐暗定时器周期 tick，每 5 秒一次
    FadeTick,
}

static QUEUE: once_cell::sync::Lazy<Queue<Event>> =
    once_cell::sync::Lazy::new(|| Queue::new(EVENT_QUEUE_LEN));

/// 获取全局事件队列的 `'static` 引用。
///
/// 首次调用时惰性创建 FreeRTOS 队列（此时调度器已启动）。
/// 多生产者（ISR / 采样任务 / 渐暗定时器回调）+ 单消费者（controller_task）。
#[inline]
pub fn queue() -> &'static Queue<Event> {
    &QUEUE
}
