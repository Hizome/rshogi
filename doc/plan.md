> [!WARNING]
> 本文档中的 GPUI 方案已废弃（2026-02-14）。  
> 当前项目实现基线为 `egui/eframe`，请勿再按 GPUI 段落执行。

这是一个基于我们之前讨论内容的完整项目开发计划文档。该计划旨在利用 Rust 和 GPUI 构建一个高性能、原生体验的将棋 GUI 应用。

Project Plan: High-Performance Native Shogi GUI (Rust + GPUI)
1. 项目愿景 (Vision)

构建一个纯 Rust 编写、GPU 加速渲染的本地桌面将棋应用。该应用将摒弃 Electron 等 Web 技术栈的臃肿，追求极致的启动速度、内存效率和渲染流畅度。核心功能是提供现代化的用户界面，并无缝接入主流的 USI (Universal Shogi Interface) 将棋引擎（如 YaneuraOu, Gikou 等）。

2. 技术栈选型 (Tech Stack)
核心语言与框架

编程语言: Rust (保证内存安全与系统级性能)

GUI 框架: GPUI (Zed 编辑器的 UI 框架，基于 Metal/Vulkan/DX12 渲染，纯 Rust)

异步运行时: Tokio 或 GPUI 内置的 Executor (用于处理非阻塞 I/O)

领域逻辑库

将棋规则: shogi crate (处理位棋盘 Bitboard、合法走法生成、胜负判定、SFEN 格式解析)

协议解析: usi-parser (或自定义解析器，用于处理 USI 协议文本流)

音频处理: rodio (用于播放落子音效)

资源格式

棋子: SVG (矢量图形，保证高分屏下的清晰度)

棋盘: PNG/JPG (高清无缝木纹贴图) + 代码绘制 (Grid 线条)

3. 架构设计 (Architecture)

采用 Model-View-Update (MVU) 或 State-Driven 的架构模式，严格分离 UI 渲染线程与引擎计算线程。

A. 数据层 (Model)

维护全局唯一的应用状态（AppState），包含：

Game State: 当前棋盘局面（Board）、手牌（Hand/Komadai）、落子记录（History/Kifu）。

UI State: 当前选中的棋子、高亮的合法落点、动画状态、视窗布局。

Engine State: 引擎连接状态、当前思考的评分（Score）、最佳变化图（PV）、思考时间。

B. 逻辑层 (Controller/Logic)

规则引擎: 封装 shogi crate，对外提供 API（如 try_move, is_mate, promote）。

引擎桥接 (Engine Bridge):

Process Manager: 使用 Rust标准库 Command 启动引擎子进程。

I/O Threads: 独立的 Reader 线程监听引擎 stdout，独立的 Writer 线程写入引擎 stdin。

Message Channel: 使用 crossbeam 或 mpsc 通道将 USI 文本消息转换为结构化事件，发送给 UI 主线程。

C. 视图层 (View - GPUI)

利用 GPUI 的声明式布局构建界面：

布局: Flexbox 风格布局，自适应窗口大小。

渲染:

棋盘: 底层渲染木纹图片，上层使用 GPUI 绘图 API 绘制抗锯齿网格线和星位点。

棋子: 加载 SVG 资源，根据坐标绝对定位或 Grid 定位。

交互: 监听鼠标点击（Click）和拖拽（Drag-and-Drop）事件，触发 Model 更新。

4. 功能模块规划 (Features Roadmap)
Phase 1: 核心原型 (Prototype)

窗口搭建: 初始化 GPUI 窗口。

棋盘渲染: 实现木纹背景与网格线的绘制。

棋子布局: 解析 SFEN 字符串，在棋盘正确位置渲染 SVG 棋子。

基础交互: 实现点击棋子高亮，点击目标格移动（仅前端逻辑，不含规则校验）。

Phase 2: 规则与状态 (Logic Integration)

接入 shogi 库: 将 UI 操作映射到核心逻辑库。

合法性校验: 限制只能走合法的步数，禁止违规移动（如二步、打步诘）。

手牌系统: 实现“驹台”的渲染，支持从驹台打入棋子。

升变处理: 当棋子进入敌阵时，弹出“成/不成”的选择对话框。

Phase 3: 引擎接入 (USI Implementation)

进程通信: 实现启动外部 .exe 或二进制引擎文件。

USI 握手: 实现 usi -> readyok -> usinewgame 流程。

思考可视化: 实时解析引擎输出的 info score 和 pv，在界面侧边栏显示当前胜率条和推荐着法。

人机对弈: 实现“人类落子 -> 发送 position -> 引擎思考 -> 接收 bestmove -> 自动落子”的闭环。

Phase 4: 完善与打磨 (Polish)

音效系统: 集成落子声（普通/提子）和读秒声。

棋谱功能: 支持导入/导出 .kif 或 .csa 格式棋谱。

设置面板: 允许用户更换棋子风格（一字驹/两字驹）、更换木纹、调整引擎思考时间。

性能优化: 减少不必要的重绘，优化 SVG 缓存。

5. 资源获取方案 (Asset Sourcing)
视觉资源

棋子 (SVG):

来源: Lishogi (GitHub 开源仓库)。

风格: 优先采用 Kinki (锦旗) 书法风格，兼顾传统与清晰度。

棋盘 (Texture):

来源: Poly Haven (CC0 材质网站)。

材质: 搜索 "Wood" 或 "Plywood"，选择浅黄色、纹理细腻的材质模拟榧木。

听觉资源

音效 (WAV):

来源: Lishogi 或 Freesound。

需求: "Click" (落子), "Capture" (吃子), "Start" (开局)。

6. 目录结构规划 (Directory Structure)
code
Text
download
content_copy
expand_less
shogi-gui/
├── assets/                 # 静态资源
│   ├── pieces/             # SVG 棋子文件 (kinki_king.svg, etc.)
│   ├── boards/             # 木纹贴图
│   └── sounds/             # WAV 音效
├── src/
│   ├── main.rs             # 程序入口，GPUI App 初始化
│   ├── logic/              # 业务逻辑层
│   │   ├── game.rs         # 封装 shogi crate 的状态机
│   │   └── format.rs       # 坐标转换与棋谱解析
│   ├── engine/             # 引擎交互层
│   │   ├── process.rs      # 进程管理与 Stdio 管道
│   │   └── usi.rs          # USI 协议解析器
│   └── ui/                 # 界面渲染层
│       ├── board.rs        # 棋盘与网格绘制
│       ├── piece.rs        # 棋子组件
│       ├── hand.rs         # 驹台组件
│       └── theme.rs        # 颜色与字体配置
└── Cargo.toml              # 依赖管理
7. 风险评估与对策 (Risks & Mitigations)

GPUI 文档稀缺:

对策: 深入阅读 Zed 编辑器的开源代码，参考其组件实现方式；在 Rust 社区积极提问。

USI 引擎兼容性:

对策: 初期仅适配最标准的 YaneuraOu 引擎，确保核心协议跑通后再做兼容性测试。

中文字体渲染:

对策: 在 GPUI 配置中显式指定支持 CJK 的字体族（如 Noto Sans CJK），防止汉字显示为“豆腐块”。
