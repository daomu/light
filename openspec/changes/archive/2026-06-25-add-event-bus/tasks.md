## 1. Event 类型

- [x] 1.1 在 `event.rs` 定义 `Event` enum 与四个变体
- [x] 1.2 派生 `Copy`/`Clone`/`Debug`/`Send`
- [x] 1.3 不派生 `PartialEq`，加注释说明原因

## 2. 全局队列

- [x] 2.1 在 `event.rs` 用 `once_cell::sync::Lazy` 声明 `static Q`
- [x] 2.2 实现 `pub fn queue() -> &'static Queue<Event>`
- [x] 2.3 容量取 `config::EVENT_QUEUE_LEN`

## 3. API 形态实测

- [x] 3.1 查 `esp_idf_svc::queue::Queue` 0.52.x 的 `send`/`recv` 签名，记录到 `design.md` 末尾
- [x] 3.2 查是否存在 `send_from_isr` 或等价方法，记录到 `design.md` 末尾
- [x] 3.3 若签名与 D4/D5 假设不符，调整本变更 spec 的字面表述，但保持"队列满丢、消费者阻塞"契约不变

## 4. 验证

- [x] 4.1 `cargo check` 通过
- [x] 4.2 临时 main：spawn 两个任务，一个 `send`，一个 `recv`，串口观察事件流通
- [x] 4.3 验证队列满时生产者 `send` 立即返回错误（不阻塞）
- [x] 4.4 验证空队列时消费者 `recv` 阻塞（不忙等）
- [x] 4.5 临时验证代码在提交前移除
