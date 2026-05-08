use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    Scene,
    Debug,
}

#[derive(Resource)]
struct DockState {
    tree: egui_tiles::Tree<Pane>,
    behavior: DockBehavior,
}

#[derive(Default)]
struct DockBehavior;

impl egui_tiles::Behavior<Pane> for DockBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Scene => "Scene".into(),
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
                ui.label("Overlapping tiles test. Left-click increments, right-click decrements.");
                ui.separator();
                ui.add_space(4.0);
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
struct Tile {
    id: egui::Id,
    local_pos: egui::Pos2,
    size: egui::Vec2,
    value: i32,
    color: egui::Color32,
}

#[derive(Resource)]
struct TileScene {
    tiles: Vec<Tile>,
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
        .insert_resource(make_tile_scene())
        .add_systems(Startup, setup)
        // IMPORTANT: run egui code inside the egui pass schedule.
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn make_dock_state() -> DockState {
    let mut tiles = egui_tiles::Tiles::default();
    let scene = tiles.insert_pane(Pane::Scene);
    let debug = tiles.insert_pane(Pane::Debug);

    let root = tiles.insert_tab_tile(vec![scene, debug]);
    let tree = egui_tiles::Tree::new("dock_tree", root, tiles);

    DockState {
        tree,
        behavior: DockBehavior::default(),
    }
}

fn make_tile_scene() -> TileScene {
    TileScene {
        tiles: vec![
            Tile {
                id: egui::Id::new("tile_a"),
                local_pos: egui::pos2(80.0, 60.0),
                size: egui::vec2(220.0, 140.0),
                value: 0,
                color: egui::Color32::from_rgb(0x3a, 0x86, 0xff),
            },
            Tile {
                id: egui::Id::new("tile_b"),
                local_pos: egui::pos2(160.0, 120.0),
                size: egui::vec2(220.0, 140.0),
                value: 0,
                color: egui::Color32::from_rgb(0xff, 0x00, 0x6e),
            },
            Tile {
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
    mut scene: ResMut<TileScene>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Click on overlapping tiles (topmost tile should change).");
            ui.separator();
            ui.monospace(format!(
                "A:{}  B:{}  C:{}",
                scene.tiles.get(0).map(|t| t.value).unwrap_or_default(),
                scene.tiles.get(1).map(|t| t.value).unwrap_or_default(),
                scene.tiles.get(2).map(|t| t.value).unwrap_or_default(),
            ));
            if ui.button("reset").clicked() {
                for t in &mut scene.tiles {
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
    draw_tiles_overlay(ctx, tiles_origin, &mut scene);
}

fn draw_tiles_overlay(ctx: &egui::Context, origin: egui::Pos2, scene: &mut TileScene) {
    // Find the click positions (if any) this frame.
    // This is intentionally "global" and does not respect widget consumption.
    let (left_click_pos, right_click_pos) = ctx.input(|i| {
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
        (left, right)
    });

    // Paint (and compute) tile rectangles in screen coordinates.
    for tile in &scene.tiles {
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

    // Topmost-only handling: scan tiles from top to bottom (reverse draw order),
    // and apply the click to the first tile whose rect contains the pointer.
    if let Some(pos) = left_click_pos {
        for i in (0..scene.tiles.len()).rev() {
            let min = origin + scene.tiles[i].local_pos.to_vec2();
            let rect = egui::Rect::from_min_size(min, scene.tiles[i].size);
            if rect.contains(pos) {
                scene.tiles[i].value += 1;
                break;
            }
        }
    }
    if let Some(pos) = right_click_pos {
        for i in (0..scene.tiles.len()).rev() {
            let min = origin + scene.tiles[i].local_pos.to_vec2();
            let rect = egui::Rect::from_min_size(min, scene.tiles[i].size);
            if rect.contains(pos) {
                scene.tiles[i].value -= 1;
                break;
            }
        }
    }
}
