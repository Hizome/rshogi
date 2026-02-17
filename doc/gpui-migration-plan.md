> [!WARNING]
> 本文档已废弃（2026-02-14）。  
> `rshogi` 当前不再使用 GPUI 方案，默认且唯一实现以 `egui/eframe` 为准。  
> 本文仅作为历史记录，不再作为开发依据。

# rshogi 迁移到 GPUI 计划（第一版）

目标：将当前 `eframe/egui` 实现平滑迁移到 `gpui`，并按需引入 `gpui-component`，优先保证棋盘交互能力不回退。

## 0. 当前进度（2026-02-14）

- 已完成：
1. 新增 `ui-gpui` feature（占位）与 `src/ui_gpui/` 目录。
2. `main.rs` 已接入 feature 分流入口，具备双入口骨架。
3. 默认 `eframe` 路径仍可 `cargo check` 通过。
4. P2 已完成最小状态流验证：GPUI 页面已接入 `GameState + Action -> reduce_game()`，可通过按钮触发状态变化并刷新。
5. P3 已完成基础静态布局：左据台-中棋盘-右据台三栏结构已在 GPUI 页面渲染（含棋盘 9x9 网格与据台 3x3 槽位显示）。
6. P4 已完成点击交互第一步：棋盘格子点击与据台槽位点击已接入 `Action` 分发，并可显示选中与合法目标高亮（拖拽待下一步）。
7. P4 已完成拖拽第二步（按下-松开模型）：支持从棋盘/据台按下开始预览，棋盘格松开提交走子或打入；拖拽中源位置会隐藏，落空则回位清空。
8. P5 核心已完成：GPUI 已接入升变弹窗覆盖层，支持 `Promote / Do not promote / Cancel`，并直接驱动 `ChoosePromotion/CancelPromotion` action。
9. P6 已完成入口切换：默认 feature 已改为 `ui-gpui`，`cargo run` 默认启动 GPUI；旧 eframe 路径保留为 `ui-egui` 备用 feature。

## 0.1 运行命令（更新）

1. 默认（GPUI）
- `cargo run`

2. 备用旧版（egui/eframe）
- `cargo run --no-default-features --features ui-egui`

- 当前阻塞：
1. 本地环境无法访问 `crates.io`，暂时无法拉取 `gpui/gpui-component` 新依赖。
2. 因此 Phase 1 的“真实 GPUI 可运行窗口”需要在可联网或本地依赖就绪后继续。

## 1. 迁移原则

1. 先跑通最小窗口和状态流，再迁移复杂交互。
2. 规则层（`src/core`）保持不动，优先替换 UI 壳层。
3. 棋盘/据台交互优先用纯 `gpui` 自绘；通用控件优先复用 `gpui-component`。
4. 分阶段并行保留旧实现，避免一次性重写导致不可运行。

## 2. 从 gpui-component 可直接借鉴的模式

基于 `gpui-component` 项目现状，可确认以下实践值得直接采用：

1. 应用初始化流程  
- `Application::new()` -> `app.run(...)`  
- 启用组件库时必须先 `gpui_component::init(cx)`  
- 顶层视图用 `Root::new(view, window, cx)`

2. 资源注入流程  
- 通过 `Application::new().with_assets(...)` 注入静态资源（图标/贴图）。
- 对你项目可用于棋盘贴图、棋子 SVG、音效等统一加载入口。

3. 组件职责划分  
- 棋盘核心区域自定义视图（强交互）  
- 设置、按钮、输入、弹窗等通用 UI 走 `gpui-component`（降开发成本）

## 3. 架构重组建议（迁移目标结构）

```text
src/
  app/
    state.rs          # 继续保留领域状态
    action.rs         # 继续保留 Action
    update.rs         # 继续保留 reducer
  core/               # 基本不动
  ui_gpui/
    app.rs            # gpui 入口、window 创建、Root 挂载
    root.rs           # 主布局（左据台-棋盘-右据台）
    board_view.rs     # 棋盘绘制与交互
    hand_view.rs      # 据台绘制与交互
    overlay_view.rs   # 升变弹窗、拖拽浮层、标注层
    assets.rs         # gpui 资源加载（with_assets）
```

说明：先新增 `ui_gpui`，旧 `ui/` 暂不删除，等功能对齐后再切换默认入口。

## 4. 分阶段执行计划

## Phase 0：基线与分支

目标：建立可回滚的迁移基线。

任务：

1. 新建迁移分支（例如 `feat/gpui-migration`）。
2. 保持当前 `eframe` 可编译可运行。
3. 在 `doc` 记录“对齐清单”（当前必须保留的能力）。

完成标准：

- `main` 仍可运行，迁移分支独立推进。

## Phase 1：最小 GPUI 外壳

目标：起一个空的 GPUI 应用并挂上 Root。

任务：

1. 在 `Cargo.toml` 增加 `gpui`，可选增加 `gpui-component`。
2. 新建 `src/ui_gpui/app.rs`，实现：
   - `Application::new()`（若使用组件则 `.with_assets(...)`）
   - `gpui_component::init(cx)`（仅使用组件时）
   - `open_window(...)`
3. 主函数支持 feature 切换：
   - `--features ui-gpui` 走 GPUI
   - 默认仍走 eframe（过渡期）

完成标准：

- `cargo run --features ui-gpui` 打开空窗口。

## Phase 2：状态流接线（不含棋盘交互）

目标：让 GPUI 页面能读写当前 `AppState`。

任务：

1. 建立 `RShogiRootView`，持有共享状态（或实体状态）引用。
2. 将 `Action -> reduce()` 流程接入 GPUI 事件。
3. 先做顶部状态栏（回合、ply、状态文案）验证单向数据流。

完成标准：

- 点击一个测试按钮可触发 Action，并驱动 UI 刷新。

## Phase 3：棋盘与据台静态渲染

目标：先对齐你已完成的布局外观。

任务：

1. 实现主布局：
   - 左侧对手据台（正方形，顶部对齐棋盘）
   - 中间棋盘
   - 右侧己方据台（正方形，底部对齐棋盘）
2. 棋盘层先实现：
   - 底图
   - 网格
   - 棋子贴图
3. 据台层先实现：
   - 3x3 槽位布局
   - 数量角标

完成标准：

- 外观基本等价当前版本，无交互回归。

## Phase 4：核心交互迁移（高优先）

目标：迁移你当前最关键的可玩能力。

任务：

1. 点击走子与合法位置高亮。
2. 拖拽走子：
   - 拖拽时原位隐藏
   - 释放合法落子/非法回位
3. 据台打入拖拽：
   - 从据台拖入棋盘
   - 同样合法性与回位逻辑

完成标准：

- 与当前 eframe 版本交互表现一致。

## Phase 5：升变与覆盖层

目标：补齐完整对局流程。

任务：

1. 迁移升变弹窗（先英文文案）。
2. 迁移拖拽浮层棋子。
3. 预留右键标注层接口（后续箭头/圈点）。

完成标准：

- 常规将棋对局流程完整可用。

## Phase 6：切换默认入口与清理

目标：完成框架切换收口。

任务：

1. 让 GPUI 成为默认入口。
2. 移除 eframe 专属代码与依赖。
3. 更新文档和开发命令。

完成标准：

- 默认 `cargo run` 启动 GPUI 版本，功能不低于迁移前。

## 5. 技术选型建议（关键决策）

1. 棋盘渲染：优先纯 `gpui` 自绘（原因：高频交互、可控性高）。
2. 通用控件：优先 `gpui-component`（按钮/输入/设置页/弹窗）。
3. 资源加载：采用 `with_assets` 统一管理静态资源路径。
4. 迁移期间保留双入口（`eframe` + `gpui`）降低中断风险。

## 6. 风险与规避

1. 编译链复杂度上升（`gpui` 依赖多）  
规避：先做最小窗口，锁定可编译版本后再扩展。

2. 输入事件语义差异（拖拽、hover、capture）  
规避：先迁移点击再迁移拖拽，逐项对照测试。

3. 资源路径变化导致贴图失效  
规避：统一 `ui_gpui/assets.rs` 做资源注册，禁止散落读取。

4. 一次性迁移范围过大  
规避：严格按 Phase 验收，未达标不进入下一阶段。

## 7. 建议的首周任务（可直接执行）

1. 建立 `ui-gpui` feature 和最小窗口（Phase 1）。
2. 建立 `RShogiRootView`，接入 `Action -> reduce`（Phase 2）。
3. 迁移静态布局（棋盘+双据台）并对齐尺寸（Phase 3）。
