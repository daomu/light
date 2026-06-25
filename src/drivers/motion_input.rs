//! AM312 GPIO 上升沿中断（技术方案 §9）。
//!
//! 在 GPIO4 上挂载输入上升沿中断，ISR 内仅向全局事件队列投递 `MotionDetected`。
//! `esp_idf_hal::task::queue::Queue::send_back` 自动检测 ISR 上下文并调用
//! `xQueueGenericSendFromISR`，故无需"ISR + 中转任务"退化路径。

use anyhow::Result;
use esp_idf_hal::gpio::{Input, InputPin, InterruptType, PinDriver, Pull};
use thiserror::Error;

use crate::event::{self, Event};
use crate::log_init;

#[derive(Debug, Error)]
pub enum MotionError {
    #[error("gpio error: {0}")]
    Esp(#[from] esp_idf_sys::EspError),
}

/// AM312 输入驱动。持有 PinDriver 的所有权，订阅中断后持续生效。
pub struct MotionInput {
    _pin: PinDriver<'static, Input>,
}

impl MotionInput {
    /// 初始化 GPIO4 为输入（上拉），配置上升沿中断，挂载回调。
    ///
    /// 回调捕获 `'static` 队列引用（通过 `once_cell::Lazy`），ISR 内直接投递事件。
    pub fn new(pin: impl InputPin + 'static) -> Result<Self, MotionError> {
        let mut pin = PinDriver::input(pin, Pull::Up)?;
        pin.set_interrupt_type(InterruptType::PosEdge)?;

        // SAFETY: 回调为 `FnMut() + Send + 'static`，仅访问全局队列（`Send + Sync`），
        // 不捕获任何非 `'static` 引用。ISR 内调用 `Queue::send_back` 是 ISR 安全的
        //（esp-idf-hal 内部自动走 `xQueueGenericSendFromISR`）。
        unsafe {
            pin.subscribe(move || {
                // 队列满：丢弃。AM312 在 Lighting 状态被忽略，丢失无业务影响。
                let _ = event::queue().send_back(Event::MotionDetected, 0);
            })?;
        }

        log_init!("AM312 irq ok, pin=GPIO{}", pin.pin());

        Ok(MotionInput { _pin: pin })
    }
}
