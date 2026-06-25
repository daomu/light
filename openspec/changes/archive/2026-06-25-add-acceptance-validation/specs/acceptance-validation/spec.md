## ADDED Requirements

### Requirement: 10 条端到端验收清单
`acceptance.md` SHALL 记录技术方案 §23 的 10 条验收标准，每条 MUST 含操作步骤、预期行为、串口日志锚点、通过/失败判据。

#### Scenario: 验收清单完整
- **WHEN** 检查 `acceptance.md`
- **THEN** §23 的 10 条全部存在，且每条四要素齐全

### Requirement: 板载灯硬件验收
第 1 条验收 SHALL 改为"板载电源灯已物理遮蔽或拆除"（用户已确认板上无可控用户 LED），MUST NOT 验收"软件拉低板载 LED"。

#### Scenario: 板载灯不破坏夜灯体验
- **WHEN** 在暗环境中观察设备
- **THEN** 板载电源灯不亮或已被物理遮蔽

### Requirement: 日志锚点对齐
每条验收的日志锚点 MUST 引用变更 1 定义的前缀宏输出（`[INIT]`/`[LUX ]`/`[EVT ]`/`[LAMP]`/`[FADE]`）。

#### Scenario: 日志锚点可观测
- **WHEN** 按验收操作步骤执行
- **THEN** 串口能观察到对应前缀的日志行，内容与预期一致

### Requirement: 渐暗验收提供两种方式
第 7/8 条渐暗验收 SHALL 同时提供"完整 20 分钟实测"与"临时缩短 tick 周期到 200ms 跑 48 秒"两种方式，验收人任选其一。

#### Scenario: 折中方式可行
- **WHEN** 临时改 `FADE_TICK_PERIOD_MS=200` 烧录运行
- **THEN** 约 48 秒内观察到 step 从 0 推进到 239 并熄灯
