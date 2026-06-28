//! AM312 GPIO 上升沿中断（技术方案 §9）。
//!
//! 在 GPIO4 上挂载输入上升沿中断，ISR 内仅向全局事件队列投递 `MotionDetected`。
//! `esp_idf_hal::task::queue::Queue::send_back` 自动检测 ISR 上下文并调用
//! `xQueueGenericSendFromISR`，故无需"ISR + 中转任务"退化路径。
//!
//! # 关键实现要点（esp-idf-hal 0.46 PinDriver 语义）
//! - `set_interrupt_type` 仅配置触发类型；`subscribe` 仅把 Rust 回调登记到
//!   `PIN_ISR_HANDLER[pin]` 静态表。二者都 **不会** 把 ISR 真正挂到 GPIO。
//!   真正的 attach 由 `PinDriver::enable_interrupt` 完成（内部调
//!   `gpio_isr_handler_add`）。漏调 → ISR 永不触发 → "黑夜移动不开灯"。
//! - esp-idf-hal 的 `handle_isr` 在每次进入时都会调 `gpio_intr_disable(pin)`
//!   以避免在电平触发下陷入死循环触发 IWDT。对 `PosEdge` 边沿触发，需要在
//!   ISR 投递完事件后调 `gpio_intr_enable(pin)` 重新使能下一次边沿（handler
//!   仍挂着，只是 per-pin 使能位被关了再开）。从 ISR 上下文调用对边沿触发
//!   是安全的：`gpio_intr_disable` 已清 status 位，重新使能后只对后续新边沿生效。
//!
//! # 调用前置条件
//! `MotionInput::new` 必须在 `event::queue()` 已被首次触发（Lazy 已初始化）
//! 之后调用。否则 ISR 回调里 `event::queue()` 会从 ISR 上下文触发
//! `once_cell::sync::Lazy` 初始化（`std::sync::Once` + `xQueueCreate` 均非
//! ISR 安全），导致 ISR 失败/卡死。

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
    /// 回调捕获 `'static` 队列引用（通过 `once_cell::Lazy`）与 pin 号（`Copy`），
    /// ISR 内直接投递事件并重新使能中断。
    ///
    /// # 前置条件
    /// 调用方必须保证 `event::queue()` 已被首次触发（Lazy 已完成初始化）。
    /// 见模块级文档。这里再次显式触发以做防御性断言：若 Lazy 未初始化，
    /// 此处会在普通任务上下文（而非 ISR）安全地完成 `xQueueCreate`。
    pub fn new(pin: impl InputPin + 'static) -> Result<Self, MotionError> {
        // 防御：确保 Lazy 已初始化。即使 main 已先行调用，重复触发也是无副作用的。
        let _ = event::queue();

        let mut pin = PinDriver::input(pin, Pull::Up)?;
        pin.set_interrupt_type(InterruptType::PosEdge)?;

        let pin_num = pin.pin();

        // SAFETY: 回调为 `FnMut() + Send + 'static`，仅访问：
        //   1. 全局队列（`Send + Sync`，`send_back` 自动走 `xQueueGenericSendFromISR`）
        //   2. 捕获的 `pin_num: u8`（`Copy`，非引用）
        // 不捕获任何非 `'static` 引用。`gpio_intr_enable` 是 ESP-IDF GPIO HAL 的
        // 寄存器级操作，对边沿触发从 ISR 内调用是安全的（见模块文档）。
        unsafe {
            pin.subscribe(move || {
                let _ = event::queue().send_back(Event::MotionDetected, 0);
                // 重新使能下一次上升沿：esp-idf-hal 的 `handle_isr` 入口已
                // `gpio_intr_disable` 清了 status 位，不重新 enable 只能收一次。
                let _ = esp_idf_sys::gpio_intr_enable(pin_num as _);
            })?;
        }

        // 真正把 ISR 挂到 GPIO4：`subscribe` 不会调 `gpio_isr_handler_add`，
        // 漏调则 ISR 永不触发。
        pin.enable_interrupt()?;

        log_init!("AM312 irq ok, pin=GPIO{}", pin.pin());

        Ok(MotionInput { _pin: pin })
    }
}
