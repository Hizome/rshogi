use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Side, Sizable, Theme, ThemeRegistry, TitleBar,
    button::{Button, ButtonVariants},
    dock::{DockArea, DockItem, DockPlacement, Panel, PanelControl, PanelEvent},
    h_flex,
    menu::{DropdownMenu as _, PopupMenuItem},
    v_flex,
};

use super::assets::{BoardWallpaper, PieceWallpaper};
use super::model::GpuiP1Shell;

const RSHOGI_DOCK_ID: &str = "rshogi-main-dock";
const RSHOGI_DOCK_VERSION: usize = 1;

pub(crate) struct GpuiDockWorkspace {
    dock_area: Entity<DockArea>,
    board: Entity<GpuiP1Shell>,
}

struct BoardDockPanel {
    focus_handle: FocusHandle,
    board: Entity<GpuiP1Shell>,
}

struct RightDockPanel {
    focus_handle: FocusHandle,
    board: Entity<GpuiP1Shell>,
    _subscription: Subscription,
}

struct BottomDockPanel {
    focus_handle: FocusHandle,
    board: Entity<GpuiP1Shell>,
    _subscription: Subscription,
}

impl GpuiDockWorkspace {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let board = cx.new(|_| GpuiP1Shell::new());
        let center_panel = cx.new(|cx| BoardDockPanel::new(board.clone(), cx));
        let right_panel = cx.new(|cx| RightDockPanel::new(board.clone(), cx));
        let bottom_panel = cx.new(|cx| BottomDockPanel::new(board.clone(), cx));

        let dock_area =
            cx.new(|cx| DockArea::new(RSHOGI_DOCK_ID, Some(RSHOGI_DOCK_VERSION), window, cx));
        let weak_dock_area = dock_area.downgrade();

        dock_area.update(cx, |dock, cx| {
            dock.set_version(RSHOGI_DOCK_VERSION, window, cx);
            dock.set_center(
                DockItem::tab(center_panel.clone(), &weak_dock_area, window, cx),
                window,
                cx,
            );
            dock.set_right_dock(
                DockItem::tab(right_panel.clone(), &weak_dock_area, window, cx),
                Some(px(320.0)),
                true,
                window,
                cx,
            );
            dock.set_bottom_dock(
                DockItem::tab(bottom_panel.clone(), &weak_dock_area, window, cx),
                Some(px(220.0)),
                true,
                window,
                cx,
            );
            dock.set_dock_collapsible(
                Edges {
                    right: true,
                    bottom: true,
                    ..Default::default()
                },
                window,
                cx,
            );
            dock.set_toggle_button_visible(false, cx);
        });

        Self { dock_area, board }
    }

    fn render_panel_toggle_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (right_open, bottom_open) = {
            let dock = self.dock_area.read(cx);
            (
                dock.is_dock_open(DockPlacement::Right, cx),
                dock.is_dock_open(DockPlacement::Bottom, cx),
            )
        };

        let right_sidebar_icon = if right_open {
            IconName::PanelRight
        } else {
            IconName::PanelRightOpen
        };
        let right_sidebar_tooltip = if right_open {
            "Hide right sidebar"
        } else {
            "Show right sidebar"
        };

        let bottom_panel_icon = if bottom_open {
            IconName::PanelBottom
        } else {
            IconName::PanelBottomOpen
        };
        let bottom_panel_tooltip = if bottom_open {
            "Hide bottom panel"
        } else {
            "Show bottom panel"
        };

        let dock_area_for_right = self.dock_area.clone();
        let dock_area_for_bottom = self.dock_area.clone();

        h_flex()
            .items_center()
            .gap_1()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .child(
                Button::new("toggle-right-sidebar")
                    .ghost()
                    .xsmall()
                    .tab_stop(false)
                    .icon(Icon::new(right_sidebar_icon).size_4())
                    .tooltip(right_sidebar_tooltip)
                    .on_click(move |_, window, cx| {
                        dock_area_for_right.update(cx, |dock, cx| {
                            dock.toggle_dock(DockPlacement::Right, window, cx);
                        });
                    }),
            )
            .child(
                Button::new("toggle-bottom-panel")
                    .ghost()
                    .xsmall()
                    .tab_stop(false)
                    .icon(Icon::new(bottom_panel_icon).size_4())
                    .tooltip(bottom_panel_tooltip)
                    .on_click(move |_, window, cx| {
                        dock_area_for_bottom.update(cx, |dock, cx| {
                            dock.toggle_dock(DockPlacement::Bottom, window, cx);
                        });
                    }),
            )
    }

    fn menu_trigger_button(cx: &App, id: &'static str, label: &'static str) -> Button {
        Button::new(id)
            .text()
            .small()
            .compact()
            .label(label)
            .text_size(px(12.0))
            .text_color(cx.theme().foreground)
    }

    fn render_file_menu(&self, cx: &App) -> impl IntoElement {
        Self::menu_trigger_button(cx, "menu-file", "File").dropdown_menu(|menu, _, _| {
            menu.item(PopupMenuItem::new("New Game").disabled(true))
                .item(PopupMenuItem::new("Open Record").disabled(true))
                .separator()
                .item(PopupMenuItem::new("Save Record").disabled(true))
        })
    }

    fn render_edit_menu(&self, cx: &App) -> impl IntoElement {
        Self::menu_trigger_button(cx, "menu-edit", "Edit").dropdown_menu(|menu, _, _| {
            menu.item(PopupMenuItem::new("Undo").disabled(true))
                .item(PopupMenuItem::new("Redo").disabled(true))
                .separator()
                .item(PopupMenuItem::new("Copy").disabled(true))
                .item(PopupMenuItem::new("Paste").disabled(true))
        })
    }

    fn render_view_menu(&self, cx: &App) -> impl IntoElement {
        let board_entity = self.board.clone();
        Self::menu_trigger_button(cx, "menu-view", "View").dropdown_menu(move |menu, window, cx| {
            let board_for_appearance = board_entity.clone();
            menu.submenu("Appearance", window, cx, move |submenu, window, cx| {
                let selected_piece_wallpaper = board_for_appearance.read(cx).piece_wallpaper();
                let selected_board_wallpaper = board_for_appearance.read(cx).board_wallpaper();
                let board_for_piece_menu = board_for_appearance.clone();
                let board_for_board_menu = board_for_appearance.clone();
                let mut submenu = submenu.check_side(Side::Left).submenu(
                    "Piece",
                    window,
                    cx,
                    move |piece_submenu, _, _| {
                        let mut piece_submenu = piece_submenu.check_side(Side::Left);
                        for wallpaper in PieceWallpaper::all() {
                            let checked = wallpaper == selected_piece_wallpaper;
                            let board = board_for_piece_menu.clone();
                            piece_submenu = piece_submenu.item(
                                PopupMenuItem::new(wallpaper.label())
                                    .checked(checked)
                                    .on_click(move |_, _, cx| {
                                        board.update(cx, |board, cx| {
                                            board.set_piece_wallpaper(wallpaper);
                                            cx.notify();
                                        });
                                    }),
                            );
                        }
                        piece_submenu
                    },
                );

                submenu = submenu.submenu("Board", window, cx, move |board_submenu, _, _| {
                    let mut board_submenu = board_submenu.check_side(Side::Left);
                    let board_for_board_menu = board_for_board_menu.clone();
                    for wallpaper in BoardWallpaper::all() {
                        let checked = wallpaper == selected_board_wallpaper;
                        let board = board_for_board_menu.clone();
                        board_submenu = board_submenu.item(
                            PopupMenuItem::new(wallpaper.label())
                                .checked(checked)
                                .on_click(move |_, _, cx| {
                                    board.update(cx, |board, cx| {
                                        board.set_board_wallpaper(wallpaper);
                                        cx.notify();
                                    });
                                }),
                        );
                    }
                    board_submenu
                });

                submenu
            })
            .submenu("Theme", window, cx, |submenu, _, cx| {
                let current_name = cx.theme().theme_name().clone();
                let mut submenu = submenu.check_side(Side::Left);
                for theme in ThemeRegistry::global(cx).sorted_themes() {
                    let theme_name = theme.name.clone();
                    let checked = theme_name == current_name;
                    submenu = submenu.item(
                        PopupMenuItem::new(theme_name.clone())
                            .checked(checked)
                            .on_click(move |_, _, cx| {
                                if let Some(theme_config) =
                                    ThemeRegistry::global(cx).themes().get(&theme_name).cloned()
                                {
                                    Theme::global_mut(cx).apply_config(&theme_config);
                                    cx.refresh_windows();
                                }
                            }),
                    );
                }
                submenu
            })
        })
    }

    fn render_game_menu(&self, cx: &App) -> impl IntoElement {
        Self::menu_trigger_button(cx, "menu-game", "Game").dropdown_menu(|menu, _, _| {
            menu.item(PopupMenuItem::new("Resign").disabled(true))
                .item(PopupMenuItem::new("Offer Draw").disabled(true))
                .separator()
                .item(PopupMenuItem::new("Flip Board").disabled(true))
        })
    }

    fn render_tools_menu(&self, cx: &App) -> impl IntoElement {
        Self::menu_trigger_button(cx, "menu-tools", "Tools").dropdown_menu(|menu, _, _| {
            menu.item(PopupMenuItem::new("Engine Settings").disabled(true))
                .item(PopupMenuItem::new("Board Preferences").disabled(true))
        })
    }

    fn render_help_menu(&self, cx: &App) -> impl IntoElement {
        Self::menu_trigger_button(cx, "menu-help", "Help").dropdown_menu(|menu, _, _| {
            menu.item(PopupMenuItem::new("Documentation").disabled(true))
                .item(PopupMenuItem::new("About").disabled(true))
        })
    }

    fn render_title_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(self.render_file_menu(cx))
                    .child(self.render_edit_menu(cx))
                    .child(self.render_view_menu(cx))
                    .child(self.render_game_menu(cx))
                    .child(self.render_tools_menu(cx))
                    .child(self.render_help_menu(cx)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .px_2()
                    .child(self.render_panel_toggle_buttons(cx)),
            )
    }
}

impl BoardDockPanel {
    fn new(board: Entity<GpuiP1Shell>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            board,
        }
    }
}

impl Panel for BoardDockPanel {
    fn panel_name(&self) -> &'static str {
        "RShogiBoardPanel"
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Board"
    }

    fn closable(&self, _: &App) -> bool {
        false
    }

    fn zoomable(&self, _: &App) -> Option<PanelControl> {
        None
    }

    fn inner_padding(&self, _: &App) -> bool {
        false
    }
}

impl EventEmitter<PanelEvent> for BoardDockPanel {}

impl Focusable for BoardDockPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BoardDockPanel {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .when(window.is_maximized(), |this| {
                this.cursor(CursorStyle::default())
            })
            .child(self.board.clone())
    }
}

impl RightDockPanel {
    fn new(board: Entity<GpuiP1Shell>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&board, |_, _, cx| cx.notify());
        Self {
            focus_handle: cx.focus_handle(),
            board,
            _subscription: subscription,
        }
    }
}

impl Panel for RightDockPanel {
    fn panel_name(&self) -> &'static str {
        "RShogiRightPanel"
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Inspector"
    }

    fn closable(&self, _: &App) -> bool {
        false
    }

    fn zoomable(&self, _: &App) -> Option<PanelControl> {
        None
    }

    fn inner_padding(&self, _: &App) -> bool {
        false
    }
}

impl EventEmitter<PanelEvent> for RightDockPanel {}

impl Focusable for RightDockPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RightDockPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .when(window.is_maximized(), |this| {
                this.cursor(CursorStyle::default())
            })
            .child(self.board.read(cx).render_right_sidebar(cx))
    }
}

impl BottomDockPanel {
    fn new(board: Entity<GpuiP1Shell>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&board, |_, _, cx| cx.notify());
        Self {
            focus_handle: cx.focus_handle(),
            board,
            _subscription: subscription,
        }
    }
}

impl Panel for BottomDockPanel {
    fn panel_name(&self) -> &'static str {
        "RShogiBottomPanel"
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Console"
    }

    fn closable(&self, _: &App) -> bool {
        false
    }

    fn zoomable(&self, _: &App) -> Option<PanelControl> {
        None
    }

    fn inner_padding(&self, _: &App) -> bool {
        false
    }
}

impl EventEmitter<PanelEvent> for BottomDockPanel {}

impl Focusable for BottomDockPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BottomDockPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .when(window.is_maximized(), |this| {
                this.cursor(CursorStyle::default())
            })
            .child(self.board.read(cx).render_bottom_panel(cx))
    }
}

impl Render for GpuiDockWorkspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .relative()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_title_bar(cx))
            .child(self.dock_area.clone())
    }
}
