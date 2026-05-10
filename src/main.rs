use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    Scene,
    D,
    E,
    F,
    Debug,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
enum ClickMode {
    OverlapAll,
    TopmostOnly,
}

#[derive(Resource)]
struct DockState {
    tree: egui_tiles::Tree<Pane>,
    behavior: DockBehavior,
}

struct DockBehavior {
    d: RoundRectPane,
    e: RoundRectPane,
    f: RoundRectPane,
}

struct RoundRectPane {
    label: &'static str,
    value: i32,
    color: egui::Color32,
}

impl RoundRectPane {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::Label::new(format!(
                "{} (pane): left-click/scroll up increments, right-click/scroll down decrements",
                self.label
            ))
            .selectable(false),
        );
        ui.add_space(8.0);

        let avail = ui.available_size();
        let desired = egui::vec2(avail.x.max(10.0), avail.y.max(10.0));
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

        if response.clicked_by(egui::PointerButton::Primary) {
            self.value += 1;
        }
        if response.clicked_by(egui::PointerButton::Secondary) {
            self.value -= 1;
        }
        if response.hovered() {
            let scroll_delta_y = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta_y > 0.0 {
                self.value += 1;
            } else if scroll_delta_y < 0.0 {
                self.value -= 1;
            }
        }

        let painter = ui.painter_at(rect);
        let bg = self.color.gamma_multiply(0.85);
        let stroke = egui::Stroke::new(2.0, egui::Color32::from_black_alpha(160));

        painter.rect_filled(rect, egui::CornerRadius::same(10), bg);
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(10),
            stroke,
            egui::StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{}", self.value),
            egui::FontId::proportional(48.0),
            egui::Color32::WHITE,
        );
    }
}

impl DockBehavior {
    fn rect_pane_mut(&mut self, pane: Pane) -> Option<&mut RoundRectPane> {
        match pane {
            Pane::D => Some(&mut self.d),
            Pane::E => Some(&mut self.e),
            Pane::F => Some(&mut self.f),
            _ => None,
        }
    }
}

impl egui_tiles::Behavior<Pane> for DockBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Scene => "Scene".into(),
            Pane::D => "D".into(),
            Pane::E => "E".into(),
            Pane::F => "F".into(),
            Pane::Debug => "Debug".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Scene => {
                ui.label(
                    "Overlapping tiles test. Left-click/scroll up increments, right-click/scroll down decrements.",
                );
                ui.separator();
                ui.add_space(4.0);
                egui_tiles::UiResponse::None
            }
            Pane::D | Pane::E | Pane::F => {
                if let Some(r) = self.rect_pane_mut(*pane) {
                    r.ui(ui);
                }
                egui_tiles::UiResponse::None
            }
            Pane::Debug => {
                ui.label("Debug pane");
                egui_tiles::UiResponse::None
            }
        }
    }
}

#[derive(Clone)]
struct RoundRect {
    id: egui::Id,
    local_pos: egui::Pos2,
    size: egui::Vec2,
    value: i32,
    color: egui::Color32,
}

#[derive(Resource)]
struct OverlayScene {
    rects: Vec<RoundRect>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "egui tiles overlap test".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(make_dock_state())
        .insert_resource(make_overlay_scene())
        .insert_resource(ClickMode::TopmostOnly)
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_click_mode)
        // IMPORTANT: run egui code inside the egui pass schedule.
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn toggle_click_mode(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<ClickMode>) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    *mode = match *mode {
        ClickMode::OverlapAll => ClickMode::TopmostOnly,
        ClickMode::TopmostOnly => ClickMode::OverlapAll,
    };
}

fn make_dock_state() -> DockState {
    let mut tiles = egui_tiles::Tiles::default();
    let scene = tiles.insert_pane(Pane::Scene);
    let d = tiles.insert_pane(Pane::D);
    let e = tiles.insert_pane(Pane::E);
    let f = tiles.insert_pane(Pane::F);
    let debug = tiles.insert_pane(Pane::Debug);

    let root = tiles.insert_tab_tile(vec![scene, d, e, f, debug]);
    let tree = egui_tiles::Tree::new("dock_tree", root, tiles);

    DockState {
        tree,
        behavior: DockBehavior {
            d: RoundRectPane {
                label: "D",
                value: 0,
                color: egui::Color32::from_rgb(0xff, 0xbe, 0x0b),
            },
            e: RoundRectPane {
                label: "E",
                value: 0,
                color: egui::Color32::from_rgb(0x8e, 0xec, 0xf5),
            },
            f: RoundRectPane {
                label: "F",
                value: 0,
                color: egui::Color32::from_rgb(0x9b, 0x5d, 0xff),
            },
        },
    }
}

fn make_overlay_scene() -> OverlayScene {
    OverlayScene {
        rects: vec![
            RoundRect {
                id: egui::Id::new("tile_a"),
                local_pos: egui::pos2(80.0, 60.0),
                size: egui::vec2(220.0, 140.0),
                value: 0,
                color: egui::Color32::from_rgb(0x3a, 0x86, 0xff),
            },
            RoundRect {
                id: egui::Id::new("tile_b"),
                local_pos: egui::pos2(160.0, 120.0),
                size: egui::vec2(220.0, 140.0),
                value: 0,
                color: egui::Color32::from_rgb(0xff, 0x00, 0x6e),
            },
            RoundRect {
                id: egui::Id::new("tile_c"),
                local_pos: egui::pos2(240.0, 180.0),
                size: egui::vec2(220.0, 140.0),
                value: 0,
                color: egui::Color32::from_rgb(0x83, 0xc5, 0xbe),
            },
        ],
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut dock: ResMut<DockState>,
    mut overlay: ResMut<OverlayScene>,
    click_mode: Res<ClickMode>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let (d_count, e_count, f_count) = {
        let b = &dock.behavior;
        (b.d.value, b.e.value, b.f.value)
    };

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Click or scroll on overlapping tiles.");
            ui.separator();
            let mode_text = match *click_mode {
                ClickMode::OverlapAll => "mode: overlap-all (SPACE toggles)",
                ClickMode::TopmostOnly => "mode: topmost-only (SPACE toggles)",
            };
            ui.monospace(mode_text);
            ui.separator();
            ui.monospace(format!(
                "A:{}  B:{}  C:{}",
                overlay.rects.get(0).map(|t| t.value).unwrap_or_default(),
                overlay.rects.get(1).map(|t| t.value).unwrap_or_default(),
                overlay.rects.get(2).map(|t| t.value).unwrap_or_default(),
            ));
            ui.separator();
            ui.monospace(format!("D:{}  E:{}  F:{}", d_count, e_count, f_count));
            if ui.button("reset").clicked() {
                for t in &mut overlay.rects {
                    t.value = 0;
                }
            }
        });
    });

    let mut tiles_origin = egui::Pos2::ZERO;
    egui::CentralPanel::default().show(ctx, |ui| {
        // IMPORTANT: only query layout/available rect *inside* the egui run.
        tiles_origin = ui.available_rect_before_wrap().min;
        let DockState { tree, behavior } = &mut *dock;
        tree.ui(behavior, ui);
    });

    // Draw the overlapping tiles over the "Scene" pane area.
    // Note: This is intentionally NOT using egui's normal widget click handling.
    // We'll do naïve global hit-testing below (so overlaps will trigger multiple updates).
    draw_overlay_rects(ctx, tiles_origin, *click_mode, &mut overlay);
}

fn draw_overlay_rects(
    ctx: &egui::Context,
    origin: egui::Pos2,
    click_mode: ClickMode,
    scene: &mut OverlayScene,
) {
    // Find the pointer events (if any) this frame.
    // This is intentionally "global" and does not respect widget consumption.
    let (left_click_pos, right_click_pos, scroll_event) = ctx.input(|i| {
        let left = i
            .pointer
            .button_clicked(egui::PointerButton::Primary)
            .then(|| i.pointer.interact_pos())
            .flatten();
        let right = i
            .pointer
            .button_clicked(egui::PointerButton::Secondary)
            .then(|| i.pointer.interact_pos())
            .flatten();

        let scroll_step = if i.raw_scroll_delta.y > 0.0 {
            Some(1)
        } else if i.raw_scroll_delta.y < 0.0 {
            Some(-1)
        } else {
            None
        };
        let scroll = scroll_step.and_then(|delta| {
            i.pointer
                .hover_pos()
                .or_else(|| i.pointer.interact_pos())
                .map(|pos| (pos, delta))
        });

        (left, right, scroll)
    });

    // Paint (and compute) tile rectangles in screen coordinates.
    for tile in &scene.rects {
        let min = origin + tile.local_pos.to_vec2();
        let rect = egui::Rect::from_min_size(min, tile.size);

        egui::Area::new(tile.id)
            .order(egui::Order::Foreground)
            .fixed_pos(min)
            .interactable(false)
            .show(ctx, |ui| {
                ui.set_min_size(tile.size);

                let bg = tile.color.gamma_multiply(0.85);
                let stroke = egui::Stroke::new(2.0, egui::Color32::from_black_alpha(160));

                let painter = ui.painter();
                painter.rect_filled(rect, egui::CornerRadius::same(8), bg);
                painter.rect_stroke(
                    rect,
                    egui::CornerRadius::same(8),
                    stroke,
                    egui::StrokeKind::Inside,
                );

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", tile.value),
                    egui::FontId::proportional(44.0),
                    egui::Color32::WHITE,
                );
            });
    }

    // Click/scroll handling:
    // - Overlap-all: apply to every tile under the pointer.
    // - Topmost-only: scan tiles from top to bottom (reverse draw order) and stop on first hit.
    if let Some(pos) = left_click_pos {
        match click_mode {
            ClickMode::OverlapAll => {
                for tile in &mut scene.rects {
                    let min = origin + tile.local_pos.to_vec2();
                    let rect = egui::Rect::from_min_size(min, tile.size);
                    if rect.contains(pos) {
                        tile.value += 1;
                    }
                }
            }
            ClickMode::TopmostOnly => {
                for i in (0..scene.rects.len()).rev() {
                    let min = origin + scene.rects[i].local_pos.to_vec2();
                    let rect = egui::Rect::from_min_size(min, scene.rects[i].size);
                    if rect.contains(pos) {
                        scene.rects[i].value += 1;
                        break;
                    }
                }
            }
        }
    }
    if let Some(pos) = right_click_pos {
        match click_mode {
            ClickMode::OverlapAll => {
                for tile in &mut scene.rects {
                    let min = origin + tile.local_pos.to_vec2();
                    let rect = egui::Rect::from_min_size(min, tile.size);
                    if rect.contains(pos) {
                        tile.value -= 1;
                    }
                }
            }
            ClickMode::TopmostOnly => {
                for i in (0..scene.rects.len()).rev() {
                    let min = origin + scene.rects[i].local_pos.to_vec2();
                    let rect = egui::Rect::from_min_size(min, scene.rects[i].size);
                    if rect.contains(pos) {
                        scene.rects[i].value -= 1;
                        break;
                    }
                }
            }
        }
    }
    if let Some((pos, delta)) = scroll_event {
        match click_mode {
            ClickMode::OverlapAll => {
                for tile in &mut scene.rects {
                    let min = origin + tile.local_pos.to_vec2();
                    let rect = egui::Rect::from_min_size(min, tile.size);
                    if rect.contains(pos) {
                        tile.value += delta;
                    }
                }
            }
            ClickMode::TopmostOnly => {
                for i in (0..scene.rects.len()).rev() {
                    let min = origin + scene.rects[i].local_pos.to_vec2();
                    let rect = egui::Rect::from_min_size(min, scene.rects[i].size);
                    if rect.contains(pos) {
                        scene.rects[i].value += delta;
                        break;
                    }
                }
            }
        }
    }
}
