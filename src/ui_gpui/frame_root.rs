use gpui::{
    AnyView, Bounds, Context, CursorStyle, Decorations, HitboxBehavior, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement, Pixels, Point, Render,
    ResizeEdge, Size, Styled as _, Window, canvas, div, point, prelude::FluentBuilder as _, px,
};
use gpui_component::ActiveTheme as _;

#[cfg(not(target_os = "linux"))]
const SHADOW_SIZE: Pixels = px(0.0);
#[cfg(target_os = "linux")]
const SHADOW_SIZE: Pixels = px(12.0);
const BORDER_SIZE: Pixels = px(1.0);

pub(crate) struct FrameRoot {
    view: AnyView,
}

impl FrameRoot {
    pub(crate) fn new(view: impl Into<AnyView>) -> Self {
        Self { view: view.into() }
    }
}

impl Render for FrameRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(cx.theme().font_size);

        let decorations = window.window_decorations();
        let can_resize_by_border = matches!(decorations, Decorations::Client { .. })
            && !window.is_maximized()
            && !window.is_fullscreen();

        window.set_client_inset(if can_resize_by_border {
            SHADOW_SIZE
        } else {
            px(0.0)
        });

        div()
            .id("window-backdrop")
            .bg(gpui::transparent_black())
            .map(|div| match decorations {
                Decorations::Server => div,
                Decorations::Client { tiling, .. } => {
                    div.bg(gpui::transparent_black())
                        .when(can_resize_by_border, |div| {
                            div.child(
                                canvas(
                                    |_bounds, window, _| {
                                        window.insert_hitbox(
                                            Bounds::new(
                                                point(px(0.0), px(0.0)),
                                                window.window_bounds().get_bounds().size,
                                            ),
                                            HitboxBehavior::Normal,
                                        )
                                    },
                                    move |_bounds, hitbox, window, _| {
                                        let mouse = window.mouse_position();
                                        let size = window.window_bounds().get_bounds().size;
                                        let Some(edge) = resize_edge(mouse, SHADOW_SIZE, size)
                                        else {
                                            return;
                                        };
                                        window.set_cursor_style(
                                            match edge {
                                                ResizeEdge::Top | ResizeEdge::Bottom => {
                                                    CursorStyle::ResizeUpDown
                                                }
                                                ResizeEdge::Left | ResizeEdge::Right => {
                                                    CursorStyle::ResizeLeftRight
                                                }
                                                ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                                                    CursorStyle::ResizeUpLeftDownRight
                                                }
                                                ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                                                    CursorStyle::ResizeUpRightDownLeft
                                                }
                                            },
                                            &hitbox,
                                        );
                                    },
                                )
                                .size_full()
                                .absolute(),
                            )
                            .when(!tiling.top, |div| div.pt(SHADOW_SIZE))
                            .when(!tiling.bottom, |div| div.pb(SHADOW_SIZE))
                            .when(!tiling.left, |div| div.pl(SHADOW_SIZE))
                            .when(!tiling.right, |div| div.pr(SHADOW_SIZE))
                            .on_mouse_down(MouseButton::Left, move |_, window, _| {
                                let size = window.window_bounds().get_bounds().size;
                                let pos = window.mouse_position();
                                if let Some(edge) = resize_edge(pos, SHADOW_SIZE, size) {
                                    window.start_window_resize(edge);
                                }
                            })
                        })
                }
            })
            .size_full()
            .child(
                div()
                    .cursor(CursorStyle::default())
                    .map(|div| match decorations {
                        Decorations::Server => div,
                        Decorations::Client { tiling } => {
                            div.when(can_resize_by_border, |div| {
                                div.border_color(cx.theme().window_border)
                                    .when(!tiling.top, |div| div.border_t(BORDER_SIZE))
                                    .when(!tiling.bottom, |div| div.border_b(BORDER_SIZE))
                                    .when(!tiling.left, |div| div.border_l(BORDER_SIZE))
                                    .when(!tiling.right, |div| div.border_r(BORDER_SIZE))
                                    .when(!tiling.is_tiled(), |div| {
                                        div.shadow(vec![gpui::BoxShadow {
                                            color: Hsla {
                                                h: 0.,
                                                s: 0.,
                                                l: 0.,
                                                a: 0.3,
                                            },
                                            blur_radius: SHADOW_SIZE / 2.,
                                            spread_radius: px(0.),
                                            offset: point(px(0.0), px(0.0)),
                                        }])
                                    })
                            })
                        }
                    })
                    .bg(gpui::transparent_black())
                    .size_full()
                    .child(
                        div()
                            .id("root")
                            .relative()
                            .size_full()
                            .font_family(cx.theme().font_family.clone())
                            .bg(cx.theme().background)
                            .text_color(cx.theme().foreground)
                            .child(self.view.clone()),
                    ),
            )
    }
}

fn resize_edge(pos: Point<Pixels>, shadow_size: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let corner_size = shadow_size + px(10.0);

    if pos.y <= corner_size && pos.x <= corner_size {
        return Some(ResizeEdge::TopLeft);
    }
    if pos.y <= corner_size && pos.x >= size.width - corner_size {
        return Some(ResizeEdge::TopRight);
    }
    if pos.y >= size.height - corner_size && pos.x <= corner_size {
        return Some(ResizeEdge::BottomLeft);
    }
    if pos.y >= size.height - corner_size && pos.x >= size.width - corner_size {
        return Some(ResizeEdge::BottomRight);
    }
    if pos.y <= shadow_size {
        return Some(ResizeEdge::Top);
    }
    if pos.y >= size.height - shadow_size {
        return Some(ResizeEdge::Bottom);
    }
    if pos.x <= shadow_size {
        return Some(ResizeEdge::Left);
    }
    if pos.x >= size.width - shadow_size {
        return Some(ResizeEdge::Right);
    }
    None
}
