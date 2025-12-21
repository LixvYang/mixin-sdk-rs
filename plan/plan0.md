---
name: mixin-sdk-rs-plan
description: Implement TODO plan to make mixin-sdk-rs fully usable, aligned with Go SDK and Safe docs
---

# Plan

目标是让 `mixin-sdk-rs` 在功能覆盖、错误处理、示例/测试上对齐 `bot-api-go-client`，以 `mixin-complete-api-reference.md` 与 `mixin-safe-api-documentation.md` 为准；并以 `mixin-api-summary-en.md`、`mixin-api-research.md` 作为快速校验表。计划按“完成一项→验证→再进入下一项”的顺序推进。

## Requirements
- API 覆盖以 `memory-bank/mixin-complete-api-reference.md` 与 `memory-bank/mixin-safe-api-documentation.md` 为准。
- 功能对齐 `bot-api-go-client` 的核心模块：auth/safe/user/message/conversation/assets/outputs/transactions/addresses/withdrawals/inscriptions/snapshots/network/fiats/url scheme 等。
- 每个 TODO 项完成后必须执行对应测试项并通过，再进入下一项。
- Rust API 设计保持异步/结果类型清晰、错误可读、示例可执行。

## Scope
- In: SDK 结构重整、模块/API 实现、必要的加密/签名、请求封装、错误与模型、示例与测试、文档对齐。
- Out: 上线部署、服务端改动、与 SDK 无关的 MCP 服务器改造。

## Files and entry points
- `mixin-sdk-rs/src/lib.rs`
- `mixin-sdk-rs/src/auth.rs`
- `mixin-sdk-rs/src/request.rs`
- `mixin-sdk-rs/src/safe.rs`
- `mixin-sdk-rs/src/user.rs`
- `mixin-sdk-rs/src/utils.rs`
- `mixin-sdk-rs/src/error.rs`
- `mixin-sdk-rs/examples/*`
- 对照：`bot-api-go-client/*.go`

## Data model / API changes
- 统一 `ApiResponse<T>`、`ApiError`、分页/offset 字段、Safe/Legacy 结构体命名。
- 安全相关：`SafeUser`、TIP 签名、ghost keys、UTXO output、交易构建/签名结构。
- 统一请求 idempotency（request_id/trace_id）与错误映射。

## Action items
[ ] 1) 基础设施对齐：梳理 `request/auth/error/utils` 与 Go SDK 对照表，补齐通用请求封装（GET/POST/PUT/DELETE + headers + JWT + idempotency）与错误类型。Test: `cargo test -p mixin-sdk-rs` 中新增 `request`/`auth` 单元测试（含 JWT 生成与签名稳定性），通过后进入下一步。  
[ ] 2) 模型层对齐：根据 `mixin-complete-api-reference.md` 建立核心数据模型（User/Asset/Address/Conversation/Message/Snapshot/Output/Transaction/Inscription 等），字段命名与 serde 对齐。Test: 为关键模型添加反序列化单测（JSON fixture → struct），至少覆盖 user/asset/output。  
[ ] 3) Safe 基础能力：实现 Safe keystore 读取、TIP 签名、ghost keys 请求；与 `mixin-safe-api-documentation.md` 中 Go/TS 示例对齐。Test: 单测覆盖 TIP 签名与 ghost keys 请求体生成；可选集成测试（若 `TEST_KEYSTORE_PATH` 存在）请求 `/safe/keys`。  
[ ] 4) 用户与关系 API：实现 `/safe/me`、`/users`、`/users/{id}`、`/search/{query}`、`/me`、`/me/preferences`、`/relationships` 等。Test: 单测 + 示例 `examples/get_me.rs`、`examples/search_user.rs` 运行成功（需要 keystore）。  
[ ] 5) 会话与消息 API：实现 conversations、participants、messages、acknowledgements、mute。Test: mock/单测 request payload；新增示例 `examples/send_message.rs`（成功返回空 data）。  
[ ] 6) 资产与网络 API：实现 `/assets`、`/assets/{id}`、`/network`、`/network/assets/top`、`/safe/assets/fetch`、`/safe/assets/{id}/fees`。Test: 单测反序列化 + 示例 `examples/list_assets.rs`。  
[ ] 7) UTXO/Outputs：实现 `/safe/outputs`、`/safe/outputs/{id}`、过滤器与分页；提供“unspent convenience”方法对齐 Go SDK。Test: request 参数构建单测 + 示例 `examples/list_outputs.rs`。  
[ ] 8) 交易与转账：实现 Safe 交易构建、签名与提交流程（包含 ghost keys、inputs/outputs 选择、tx hash/extra），对齐 `mixin-safe-api-documentation.md` 交易流程。Test: 单测覆盖交易签名与 hash；集成示例 `examples/transfer.rs`（仅在配置 keystore 时运行）。  
[ ] 9) Address/Withdrawal：实现地址增删/查询与提现流程（TIP 签名）。Test: request 构造单测 + 示例 `examples/create_address.rs`。  
[ ] 10) Snapshot/Invoice/Inscription/Collectibles：按 complete reference 完成剩余 API。Test: 每个模块至少 1 个反序列化单测 + 1 个示例。  
[ ] 11) URL Scheme/Utilities：补齐 url scheme 生成、UUID/trace helpers，与 Go SDK 测试对齐。Test: 纯函数单测。  
[ ] 12) 文档与示例完善：更新 `mixin-sdk-rs/README.md` 与 `examples/` 索引，标注 Safe 与 Legacy 范围。Test: `cargo test` 与关键示例可运行性检查。  

## Testing and validation
- 统一 `cargo test`（单元/反序列化/纯函数）。
- 可选集成测试：依赖 `TEST_KEYSTORE_PATH`。
- 示例运行：`cargo run --example <name> --all-features`。
- 对照测试：与 `bot-api-go-client` 同名功能的行为一致性检查。

## Risks and edge cases
- Safe 交易构建涉及 UTXO 选择与费用计算，需严格对照文档示例。
- API 字段易变或文档差异：以 `mixin-complete-api-reference.md` / `mixin-safe-api-documentation.md` 为准。
- 集成测试依赖真实 keystore，需在 CI 中做条件跳过。

## Open questions
- 是否需要保留 Legacy PIN 相关 API（Go SDK 部分仍含 legacy）？
- 计划对齐到 Go SDK 的哪些“非核心”模块（monitor/report、cli/demo）？
