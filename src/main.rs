use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReproMode {
    StaleOwnerPreUpdate,
    CurrentPointerHitTest,
}

impl ReproMode {
    fn label(self) -> &'static str {
        match self {
            ReproMode::StaleOwnerPreUpdate => "BUG: stale owner in PreUpdate",
            ReproMode::CurrentPointerHitTest => "FIX: current pointer hit-test",
        }
    }

    fn toggle(self) -> Self {
        match self {
            ReproMode::StaleOwnerPreUpdate => ReproMode::CurrentPointerHitTest,
            ReproMode::CurrentPointerHitTest => ReproMode::StaleOwnerPreUpdate,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum InputOwner {
    #[default]
    None,
    Scene,
    DPane,
}

impl InputOwner {
    fn label(self) -> &'static str {
        match self {
            InputOwner::None => "none",
            InputOwner::Scene => "scene",
            InputOwner::DPane => "D",
        }
    }

    fn color(self) -> egui::Color32 {
        match self {
            InputOwner::None => egui::Color32::GRAY,
            InputOwner::Scene => egui::Color32::from_rgb(65, 160, 105),
            InputOwner::DPane => egui::Color32::from_rgb(235, 130, 65),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct OwnerRegion {
    rect: egui::Rect,
    owner: InputOwner,
    priority: u8,
}

#[derive(Clone, Debug)]
struct SceneTile {
    label: &'static str,
    local_pos: egui::Pos2,
    size: egui::Vec2,
    value: i32,
    color: egui::Color32,
}

#[derive(Resource, Debug)]
struct SceneTiles {
    items: Vec<SceneTile>,
}

impl Default for SceneTiles {
    fn default() -> Self {
        Self {
            items: vec![
                SceneTile {
                    label: "A",
                    local_pos: egui::pos2(80.0, 70.0),
                    size: egui::vec2(270.0, 170.0),
                    value: 0,
                    color: egui::Color32::from_rgb(0x3a, 0x86, 0xff),
                },
                SceneTile {
                    label: "B",
                    local_pos: egui::pos2(185.0, 145.0),
                    size: egui::vec2(270.0, 170.0),
                    value: 0,
                    color: egui::Color32::from_rgb(0xff, 0x00, 0x6e),
                },
                SceneTile {
                    label: "C",
                    local_pos: egui::pos2(290.0, 220.0),
                    size: egui::vec2(270.0, 170.0),
                    value: 0,
                    color: egui::Color32::from_rgb(0x83, 0xc5, 0xbe),
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ScreenTile {
    rect: egui::Rect,
}

#[derive(Clone, Copy, Debug)]
enum Banner {
    Bug(&'static str),
    Blocked,
    DHandled,
}

#[derive(Resource, Debug)]
struct ReproState {
    mode: ReproMode,
    owner_regions_from_last_egui_pass: Vec<OwnerRegion>,
    tile_rects_from_last_egui_pass: Vec<ScreenTile>,
    owner_from_last_egui_pass: InputOwner,
    pointer_owner_now: InputOwner,
    pointer_owner_seen_by_preupdate: InputOwner,
    owner_used_by_preupdate: InputOwner,
    d_value: i32,
    frame: u64,
    banner: Option<Banner>,
    last_event: String,
}

impl Default for ReproState {
    fn default() -> Self {
        Self {
            mode: ReproMode::StaleOwnerPreUpdate,
            owner_regions_from_last_egui_pass: Vec::new(),
            tile_rects_from_last_egui_pass: Vec::new(),
            owner_from_last_egui_pass: InputOwner::None,
            pointer_owner_now: InputOwner::None,
            pointer_owner_seen_by_preupdate: InputOwner::None,
            owner_used_by_preupdate: InputOwner::None,
            d_value: 0,
            frame: 0,
            banner: None,
            last_event: "Hold left mouse on C, then drag into D.".to_string(),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PreUpdate owner lag repro".to_string(),
                resolution: (1180, 760).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(SceneTiles::default())
        .insert_resource(ReproState::default())
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, scene_input_in_preupdate)
        .add_systems(Update, keyboard_shortcuts)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut tiles: ResMut<SceneTiles>,
    mut repro: ResMut<ReproState>,
) {
    if keys.just_pressed(KeyCode::KeyL) || keys.just_pressed(KeyCode::Space) {
        repro.mode = repro.mode.toggle();
        repro.banner = None;
        repro.last_event = format!("Mode: {}", repro.mode.label());
    }

    if keys.just_pressed(KeyCode::KeyR) {
        reset_counts(&mut tiles, &mut repro);
    }
}

fn scene_input_in_preupdate(
    mut tiles: ResMut<SceneTiles>,
    mut repro: ResMut<ReproState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    repro.frame += 1;

    let pointer = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .map(|pos| egui::pos2(pos.x, pos.y));

    let scroll_delta = mouse_wheel.read().map(|event| event.y).sum::<f32>();
    let click_delta = if mouse_buttons.just_pressed(MouseButton::Left) {
        1
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        -1
    } else {
        0
    };
    let input_delta = if scroll_delta > 0.0 {
        1
    } else if scroll_delta < 0.0 {
        -1
    } else {
        click_delta
    };

    let pointer_owner_hit_test = pointer
        .map(|pos| owner_at(&repro.owner_regions_from_last_egui_pass, pos))
        .unwrap_or_default();
    let entered_d_while_holding = mouse_buttons.pressed(MouseButton::Left)
        && repro.pointer_owner_seen_by_preupdate == InputOwner::Scene
        && pointer_owner_hit_test == InputOwner::DPane;

    let owner_used = match repro.mode {
        ReproMode::StaleOwnerPreUpdate => repro.owner_from_last_egui_pass,
        ReproMode::CurrentPointerHitTest => pointer_owner_hit_test,
    };

    repro.pointer_owner_now = pointer_owner_hit_test;
    repro.pointer_owner_seen_by_preupdate = pointer_owner_hit_test;
    repro.owner_used_by_preupdate = owner_used;

    let input_delta = if input_delta != 0 {
        input_delta
    } else if entered_d_while_holding {
        1
    } else {
        0
    };

    if input_delta == 0 {
        return;
    }

    if owner_used != InputOwner::Scene {
        repro.banner = Some(Banner::Blocked);
        repro.last_event = format!(
            "Frame {}: D blocked the scene input. PreUpdate used owner = {}.",
            repro.frame,
            owner_used.label()
        );
        return;
    }

    let Some(pos) = pointer else {
        return;
    };
    let Some(tile_index) = topmost_tile_at(&repro.tile_rects_from_last_egui_pass, pos) else {
        return;
    };

    tiles.items[tile_index].value += input_delta;
    let tile_label = tiles.items[tile_index].label;

    if pointer_owner_hit_test == InputOwner::DPane {
        repro.banner = Some(Banner::Bug(tile_label));
        repro.last_event = format!(
            "Frame {}: {} changed even though the pointer hit D.",
            repro.frame, tile_label
        );
    } else {
        repro.banner = None;
        repro.last_event = format!(
            "Frame {}: {} changed from scene input.",
            repro.frame, tile_label
        );
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut tiles: ResMut<SceneTiles>,
    mut repro: ResMut<ReproState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    top_bar(ctx, &mut tiles, &mut repro);

    let mut owner_regions = Vec::new();
    let mut tile_rects = Vec::new();
    let mut d_rect = egui::Rect::NOTHING;

    egui::CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_rect_before_wrap();
        let scene_rect = egui::Rect::from_min_max(
            available.min + egui::vec2(44.0, 34.0),
            available.max - egui::vec2(44.0, 42.0),
        );
        let tile_origin = scene_rect.min + egui::vec2(26.0, 52.0);

        tile_rects = tiles
            .items
            .iter()
            .map(|tile| ScreenTile {
                rect: egui::Rect::from_min_size(tile_origin + tile.local_pos.to_vec2(), tile.size),
            })
            .collect();

        let c_rect = tile_rects[2].rect;
        d_rect = egui::Rect::from_min_size(
            c_rect.min + egui::vec2(126.0, 48.0),
            egui::vec2(132.0, 82.0),
        );

        owner_regions = vec![
            OwnerRegion {
                rect: scene_rect,
                owner: InputOwner::Scene,
                priority: 0,
            },
            OwnerRegion {
                rect: d_rect,
                owner: InputOwner::DPane,
                priority: 10,
            },
        ];

        let pointer = ctx.input(|input| input.pointer.latest_pos());
        let owner_now = pointer
            .map(|pos| owner_at(&owner_regions, pos))
            .unwrap_or_default();
        repro.pointer_owner_now = owner_now;

        paint_scene(ui, scene_rect, &tile_rects, &tiles, &repro);
    });

    d_pane(ctx, d_rect, &mut repro);

    repro.owner_from_last_egui_pass = repro.pointer_owner_now;
    repro.owner_regions_from_last_egui_pass = owner_regions;
    repro.tile_rects_from_last_egui_pass = tile_rects;
}

fn top_bar(ctx: &egui::Context, tiles: &mut SceneTiles, repro: &mut ReproState) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("toggle mode (L / Space)").clicked() {
                repro.mode = repro.mode.toggle();
                repro.banner = None;
                repro.last_event = format!("Mode: {}", repro.mode.label());
            }
            if ui.button("reset (R)").clicked() {
                reset_counts(tiles, repro);
            }
            ui.separator();
            ui.strong(repro.mode.label());
            ui.separator();
            ui.monospace("A/B/C: scene behind, topmost-only");
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            owner_chip(ui, "pointer over", repro.pointer_owner_now);
            owner_chip(ui, "PreUpdate used", repro.owner_used_by_preupdate);
            ui.separator();
            ui.monospace(format!(
                "A:{}  B:{}  C:{}  D:{}",
                tiles.items[0].value, tiles.items[1].value, tiles.items[2].value, repro.d_value
            ));
        });
        ui.add_space(6.0);
        ui.monospace(&repro.last_event);
        ui.add_space(8.0);
    });
}

fn owner_chip(ui: &mut egui::Ui, title: &str, owner: InputOwner) {
    let color = owner.color();
    egui::Frame::new()
        .fill(color.gamma_multiply(0.14))
        .stroke(egui::Stroke::new(1.0, color))
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.monospace(format!("{title}: {}", owner.label()));
        });
}

fn paint_scene(
    ui: &mut egui::Ui,
    scene_rect: egui::Rect,
    tile_rects: &[ScreenTile],
    tiles: &SceneTiles,
    repro: &ReproState,
) {
    let painter = ui.painter();

    painter.rect_filled(
        scene_rect,
        egui::CornerRadius::same(8),
        egui::Color32::from_rgb(24, 29, 38),
    );
    painter.rect_stroke(
        scene_rect,
        egui::CornerRadius::same(8),
        egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 105, 135)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        scene_rect.left_top() + egui::vec2(22.0, 18.0),
        egui::Align2::LEFT_TOP,
        "Test: hold left mouse on C, then drag into D",
        egui::FontId::monospace(17.0),
        egui::Color32::WHITE,
    );

    for (index, tile) in tiles.items.iter().enumerate() {
        let rect = tile_rects[index].rect;
        painter.rect_filled(rect, egui::CornerRadius::same(8), tile.color);
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(8),
            egui::Stroke::new(2.0, egui::Color32::from_black_alpha(170)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            rect.left_center() + egui::vec2(24.0, 0.0),
            egui::Align2::LEFT_CENTER,
            format!("{} {}", tile.label, tile.value),
            egui::FontId::monospace(48.0),
            egui::Color32::WHITE,
        );
    }

    if let Some(banner) = repro.banner {
        let (text, color) = match banner {
            Banner::Bug(tile) => (
                format!("BUG: {tile} changed while pointer was over D"),
                egui::Color32::from_rgb(210, 48, 48),
            ),
            Banner::Blocked => (
                "OK: D blocked the scene input".to_string(),
                egui::Color32::from_rgb(55, 130, 85),
            ),
            Banner::DHandled => (
                "D handled the UI event".to_string(),
                egui::Color32::from_rgb(55, 110, 180),
            ),
        };
        let banner_rect = egui::Rect::from_min_size(
            scene_rect.left_bottom() + egui::vec2(22.0, -74.0),
            egui::vec2(scene_rect.width() - 44.0, 50.0),
        );
        painter.rect_filled(banner_rect, egui::CornerRadius::same(6), color);
        painter.text(
            banner_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(20.0),
            egui::Color32::WHITE,
        );
    }
}

fn d_pane(ctx: &egui::Context, rect: egui::Rect, repro: &mut ReproState) {
    egui::Area::new(egui::Id::new("ui_pane_d"))
        .order(egui::Order::Tooltip)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            let (response_rect, response) =
                ui.allocate_exact_size(rect.size(), egui::Sense::click());

            let scroll_delta = ui.input(|input| input.raw_scroll_delta.y);
            let mut handled = false;
            if response.clicked_by(egui::PointerButton::Primary) {
                repro.d_value += 1;
                handled = true;
            } else if response.clicked_by(egui::PointerButton::Secondary) {
                repro.d_value -= 1;
                handled = true;
            }

            if response.hovered() {
                if scroll_delta > 0.0 {
                    repro.d_value += 1;
                    handled = true;
                } else if scroll_delta < 0.0 {
                    repro.d_value -= 1;
                    handled = true;
                }
            }

            if handled && repro.banner.is_none() {
                repro.banner = Some(Banner::DHandled);
            }

            let painter = ui.painter();
            painter.rect_filled(
                response_rect,
                egui::CornerRadius::same(8),
                egui::Color32::from_rgb(52, 41, 30),
            );
            painter.rect_stroke(
                response_rect,
                egui::CornerRadius::same(8),
                egui::Stroke::new(2.0, egui::Color32::from_rgb(250, 170, 70)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                response_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("UI pane D\nD: {}", repro.d_value),
                egui::FontId::monospace(22.0),
                egui::Color32::WHITE,
            );
        });
}

fn reset_counts(tiles: &mut SceneTiles, repro: &mut ReproState) {
    for tile in &mut tiles.items {
        tile.value = 0;
    }
    repro.d_value = 0;
    repro.banner = None;
    repro.last_event = "Reset A/B/C/D.".to_string();
}

fn owner_at(regions: &[OwnerRegion], pos: egui::Pos2) -> InputOwner {
    regions
        .iter()
        .filter(|region| region.rect.contains(pos))
        .max_by_key(|region| region.priority)
        .map(|region| region.owner)
        .unwrap_or_default()
}

fn topmost_tile_at(tile_rects: &[ScreenTile], pos: egui::Pos2) -> Option<usize> {
    (0..tile_rects.len())
        .rev()
        .find(|&index| tile_rects[index].rect.contains(pos))
}
