> [!WARNING]
> 本文档中涉及 GPUI 的建议已废弃（2026-02-14）。  
> 当前仅以 `egui/eframe` 路线为有效实现路线。

# rshogi 项目讨论总结（架构、参考映射、引擎分析）

本文整理了当前为止关于 `rshogi` 的主要讨论结论，作为 `doc/plan.md` 的补充设计文档。

## 1. 当前项目状态

- 当前仓库 `rshogi` 仅有 `doc/plan.md`，还没有代码实现。
- 现阶段最关键的是先确定“可执行的实现顺序”和“技术边界”，避免早期返工。

## 2. 总体方向结论

`Rust + GPUI` 方向可行，建议继续推进。核心建议是：

- 借鉴 `lishogi` 的“棋局状态流 + USI 协议状态机 + 分析 UI 交互”。
- 借鉴 `gpui-component` 的“GPUI 状态管理和异步更新模式”。
- 不直接搬运两边代码，重点复用设计模式与模块边界。

## 3. 推荐架构（rshogi）

建议采用四层结构：

1. `core`：棋局规则与状态（SFEN、合法走子、手牌、升变、历史）
2. `engine`：USI 引擎桥接（子进程、协议、事件通道）
3. `ui`：GPUI 视图与交互（棋盘、驹台、分析栏、设置）
4. `assets`：棋子/棋盘/音效资源与缓存

建议使用单向数据流：

- `AppState + Action + update()`
- UI 只发动作，不直接改棋局与引擎状态
- 引擎层通过 `EngineCommand / EngineEvent` 与 UI 解耦

## 4. 里程碑建议（调整后）

相比原 `plan.md`，建议把“规则闭环”和“引擎闭环”前置：

1. `M1`：SFEN 加载 + 棋盘显示 + 合法走子（含吃子/轮次）
2. `M2`：驹台打入 + 升变选择
3. `M3`：USI 本地引擎接入，完成人机闭环
4. `M4`：分析面板、棋谱、音效、设置和性能优化

关键点：不要把 Phase 1 仅做成“前端假移动”。

## 5. 从 lishogi 可直接借鉴的实现思路

重点文件（已分析）：

- `ui/round/src/ground.ts`
- `ui/round/src/ctrl.ts`
- `ui/ceval/src/protocol.ts`
- `ui/ceval/src/view.ts`
- `ui/analyse/src/ctrl.ts`

可借鉴点：

- 局面驱动的棋盘配置生成（非命令式逐块修改）
- 走子/打入/升变的统一事件入口
- 变体兼容时的协议转换层（fairy 转换）
- 分析 UI 与引擎协议分层（`ctrl`、`protocol`、`worker`、`view`）

## 6. lishogi 的引擎接入方式（核心结论）

lishogi 不是单一路径，而是三条引擎链路并存：

### 6.1 对局 AI 落子（服务端 shoginet）

- 对局中玩家走子后，服务端 round 逻辑判断是否请求 AI。
- shoginet 创建 `Work.Move` 任务，外部引擎客户端领取并返回 bestmove。
- 返回后通过 socket/actor 回推对局，完成 AI 落子。

相关文件：

- `modules/round/src/main/Player.scala`
- `modules/shoginet/src/main/Player.scala`
- `modules/shoginet/src/main/ShoginetApi.scala`

### 6.2 本地分析（前端 WASM 引擎）

- 分析页通过 `ui/ceval` 模块启动本地 WASM 引擎（不是本机子进程）。
- `protocol.ts` 实现 USI 生命周期：`usi -> isready -> usinewgame -> position -> go -> info/bestmove`。
- 根据局面和变体，选择 `YaneuraOu` 或 `Fairy Stockfish`。

相关文件：

- `ui/ceval/src/ctrl.ts`
- `ui/ceval/src/worker.ts`
- `ui/ceval/src/protocol.ts`
- `ui/shogi/src/engine-name.ts`

### 6.3 云评估缓存（eval cache）

- 分析页会发送 `evalGet/evalPut`，服务端返回 `evalHit`。
- 同时服务器完整分析进度会通过 `analysisProgress` 推送到前端树。

相关文件：

- `ui/analyse/src/eval-cache.ts`
- `ui/analyse/src/socket.ts`
- `modules/evalCache/src/main/EvalCacheSocketHandler.scala`
- `modules/round/src/main/RoundDuct.scala`

## 7. lishogi 的“引擎分析 UI”做了什么

核心界面能力（`ui/ceval/src/view.ts`）：

- 实时评估条（gauge）
- 当前分数（cp / mate）
- 深度、进度、nodes/s（kN/s）信息
- MultiPV 多候选着展示
- Threat mode（威胁分析模式）
- Go deeper（继续深挖）
- PV 悬停预览小棋盘、滚轮浏览 PV、按 PV 快速试走

控制项（`ui/analyse/src/action-menu.ts`）：

- 开关分析、无限分析
- MultiPV、Threads、Hash
- NNUE 开关、EnteringKingRule、Fixed Memory

## 8. 这些能力对应到 rshogi 应如何实现

建议一一映射，不照搬网页实现：

1. `ceval worker` 替换为本地 USI 子进程 actor
2. 保留 `protocol` 状态机思想，继续使用结构化事件
3. 保留 `view` 的信息架构（评估条 + PV + mini-board + 参数面板）
4. 先支持标准将棋（YaneuraOu），再逐步扩展到变体与转换层

## 9. 与 gpui-component 的结合建议

已验证可借鉴的 GPUI 模式：

- `Entity` 管理可变状态
- `cx.spawn` 做后台循环与异步任务
- `cx.notify()` 驱动局部刷新
- 顶层 `Root` 管理全局 UI 层（弹窗、通知、覆盖层）
- 资源打包可参考 `examples/app_assets`

相关文件：

- `examples/system_monitor/src/main.rs`
- `crates/ui/src/root.rs`
- `examples/app_assets/src/main.rs`

## 10. 风险与优先级

优先风险：

1. GPUI 文档和生态相对稀疏（需依赖源码理解）
2. USI 引擎兼容差异（不同引擎 option 行为不同）
3. UI 与引擎线程耦合导致卡顿（必须事件化和异步化）

应对策略：

- 先锁定单引擎（YaneuraOu）打通闭环
- 协议层做“最小可用 + 可扩展字段”而非一次性全量实现
- 每个里程碑都保留可运行状态并验证端到端流程

## 11. 推荐下一步（可直接执行）

1. 初始化工程骨架（`src/core`, `src/engine`, `src/ui`, `assets`）
2. 建立 `AppState` 与 `Action`，先跑通无引擎棋盘合法走子
3. 增加 `engine` actor，先实现最小 USI 循环
4. 接入分析栏最小版（分数 + 主 PV）

---

该文档来源于当前讨论与本地参考仓库代码扫描结果，后续实现可按里程碑持续细化。
