use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContext,
};

use super::{
    display::{ToggleViewModeEvent, ViewMode},
    nasa_horizons::{NasaBodyAddition, SpawnNasaBodyRequest},
    simulation::{
        systems::ToggleSpaceSimulationStateEvent, SpaceSimulation, SpaceSimulationParams,
        SpaceSimulationState,
    },
};

#[derive(Resource)]
pub struct ShowUI {
    pub value: bool,
}

impl Default for ShowUI {
    fn default() -> Self {
        Self { value: true }
    }
}

pub fn ui_system(
    mut ctx: ResMut<EguiContext>,
    mut show_ui: Local<ShowUI>,
    view_mode: Res<State<ViewMode>>,
    mut view_mode_ev: EventWriter<ToggleViewModeEvent>,
    space_simulation_state: Res<State<SpaceSimulationState>>,
    mut space_simulation_state_ev: EventWriter<ToggleSpaceSimulationStateEvent>,
    keyboard: Res<Input<ScanCode>>,
    mut nasa_body_request_ev: EventWriter<SpawnNasaBodyRequest>,
    mut nasa_body_addition_ev: EventWriter<NasaBodyAddition>,
    mut space_simulation_params: ResMut<SpaceSimulationParams>,
    mut space_simulation: ResMut<SpaceSimulation>,
) {
    // ESC
    if keyboard.just_pressed(ScanCode(1)) {
        show_ui.value = !show_ui.value;
    }

    let ctx = ctx.ctx_mut();

    ctx.set_style({
        let mut style = ctx.style().as_ref().clone();
        style.visuals.panel_fill = Color32::BLACK;
        style.visuals.widgets.inactive.bg_fill = Color32::BLACK;
        style
    });

    if show_ui.value {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Управление:")
                        .heading()
                        .color(Color32::LIGHT_BLUE),
                );
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::YELLOW, "ESC");
                        ui.colored_label(Color32::WHITE, "- закрыть это окно");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::YELLOW, "Q");
                        ui.colored_label(Color32::WHITE, "- сменить режим камеры");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::YELLOW, "Space");
                        ui.colored_label(Color32::WHITE, "- запустить/остановить симуляцию");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::YELLOW, "ЛКМ");
                        ui.colored_label(Color32::WHITE, "- выделение");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::YELLOW, "ЛКМ + CRTL");
                        ui.colored_label(Color32::WHITE, "- вторичное выделение");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::YELLOW, "W");
                        ui.colored_label(Color32::WHITE, "- сменить точку отсчёта орбиты");
                    });
                });

                ui.separator();

                ui.label(
                    RichText::new("Действия:")
                        .heading()
                        .color(Color32::LIGHT_BLUE),
                );
                ui.group(|ui| {
                    if ui
                        .button(
                            RichText::new(format!(
                                "Режим камеры: {}",
                                match view_mode.current() {
                                    ViewMode::Realistic => "Правдоподобный",
                                    ViewMode::Schematic => "Схематичный",
                                }
                            ))
                            .color(Color32::LIGHT_YELLOW),
                        )
                        .clicked()
                    {
                        view_mode_ev.send(ToggleViewModeEvent);
                    }
                    if ui
                        .button(
                            RichText::new(format!(
                                "Состояние симуляции: {}",
                                match space_simulation_state.current() {
                                    SpaceSimulationState::Running => "Запущена",
                                    SpaceSimulationState::Stopped => "Остановлена",
                                }
                            ))
                            .color(Color32::LIGHT_YELLOW),
                        )
                        .clicked()
                    {
                        space_simulation_state_ev.send(ToggleSpaceSimulationStateEvent);
                    }
                });

                ui.separator();

                ui.label(
                    RichText::new("Вставка тел локально:")
                        .heading()
                        .color(Color32::LIGHT_BLUE),
                );
                ui.group(|ui| {
                    let mut insert_bodies_local = |filename: &str| {
                        let bodies: Vec<NasaBodyAddition> =
                            serde_json::from_str(&std::fs::read_to_string(filename).unwrap())
                                .unwrap();
                        let earliest_time = bodies
                            .iter()
                            .min_by(|x, y| x.date.cmp(&y.date))
                            .unwrap()
                            .date;
                        space_simulation.time = earliest_time;
                        nasa_body_addition_ev.send_batch(bodies);
                    };

                    if ui
                        .button(RichText::new("Основные тела").color(Color32::LIGHT_YELLOW))
                        .clicked()
                    {
                        insert_bodies_local("./assets/bodies/major-bodies.json");
                    }
                    if ui
                        .button(RichText::new("Все доступные тела").color(Color32::LIGHT_YELLOW))
                        .clicked()
                    {
                        insert_bodies_local("./assets/bodies/all-bodies.json");
                    }
                });

                ui.separator();

                ui.label(
                    RichText::new("Вставка тел с NASA:")
                        .heading()
                        .color(Color32::LIGHT_BLUE),
                );
                ui.group(|ui| {
                    let mut insert_bodies_nasa = |filepath: &str| {
                        nasa_body_request_ev.send_batch(
                            std::fs::read_to_string(filepath)
                                .unwrap()
                                .split(";")
                                .map(|name| crate::space::nasa_horizons::SpawnNasaBodyRequest {
                                    date: chrono::Utc::now(),
                                    name: name.into(),
                                }),
                        );
                    };

                    if ui
                        .button(RichText::new("Основные тела").color(Color32::LIGHT_YELLOW))
                        .clicked()
                    {
                        insert_bodies_nasa("./assets/bodies/major-bodies-query.txt");
                    }
                    if ui
                        .button(RichText::new("Все доступные тела").color(Color32::LIGHT_YELLOW))
                        .clicked()
                    {
                        insert_bodies_nasa("./assets/bodies/all-bodies-query.txt");
                    }
                });

                ui.separator();

                ui.label(
                    RichText::new("Параметры:")
                        .heading()
                        .color(Color32::LIGHT_BLUE),
                );
                ui.group(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut space_simulation_params.speed)
                            .speed(1.0)
                            .clamp_range(1.0..=f64::INFINITY)
                            .prefix("скорость симуляции: ")
                            .suffix(" сек."),
                    );

                    ui.add(
                        egui::DragValue::new(&mut space_simulation_params.percision)
                            .speed(1)
                            .clamp_range(1..=32)
                            .prefix("точность симуляции: "),
                    );
                });
            });
    }

    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.label(space_simulation.time.to_string());
        });
    });
}

pub struct SpaceUIPlugin;

impl Plugin for SpaceUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui_system);
    }
}
