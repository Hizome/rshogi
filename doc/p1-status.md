# P1 当前实现状态

## 已完成

- Rust 工程初始化并可编译运行
- 使用 `shogi` crate 完成规则内核接入
- 加载初始 SFEN 局面
- 9x9 棋盘渲染（桌面窗口）
- 棋盘贴图渲染（`assets/boards/lishogi/kaya1.jpg`）
- SVG 棋子渲染（`assets/pieces/standard/western/*.svg`）
- 点击选子 + 合法落点高亮
- 仅允许合法走子
- 吃子后手牌自动更新并显示

## 运行方式

```bash
cargo run
```

## 当前边界（故意留到下一阶段）

- 未实现驹台打入（Drop）
- 未实现升变选择弹窗（当前策略是可选升变时默认不升，必要时自动用升变着）
- 音效资源已导入但尚未接入播放

## 静态资源计划

- 已预留目录：`assets/pieces/`, `assets/boards/`, `assets/sounds/`
- 资源来源建议见：`assets/README.md`
