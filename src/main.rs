#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping}, prelude::*, window::PresentMode};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use visualizer::AudioVisualizerPlugin;

mod visualizer;
mod audio_data;
mod ui;

use audio_data::AudioDataPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(UiPlugin)
        .add_plugins(AudioDataPlugin)
        .add_plugins(AudioVisualizerPlugin)
        .init_resource::<AdvancedSettings>()
        .add_event::<AdvancedSettingsChangeEvent>()
        .add_systems(Startup, (setup_camera, setup_fps_counter))
        .add_systems(Update, measure_fps.run_if(|advanced_settings: Res<AdvancedSettings>| advanced_settings.show_fps))
        .add_systems(Update, update_advanced_settings.run_if(on_event::<AdvancedSettingsChangeEvent>()))
        .run();
}

#[derive(Component)]
struct FpsCounter;

#[derive(Resource)]
pub struct AdvancedSettings {
    pub vsync: bool,
    pub show_fps: bool
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        AdvancedSettings {
            vsync: true,
            show_fps: false
        }
    }
}

#[derive(Event)]
pub struct AdvancedSettingsChangeEvent;

fn update_advanced_settings(
    mut fps_counter_query: Query<&mut Visibility, With<FpsCounter>>,
    mut window_query: Query<&mut Window>,
    advanced_settings: Res<AdvancedSettings>
) {
    let mut window = window_query.single_mut();
    window.present_mode = if advanced_settings.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };

    let mut fps_counter_visibility = fps_counter_query.get_single_mut().unwrap();

    if advanced_settings.show_fps {
        *fps_counter_visibility = Visibility::Visible;
    } else {
        *fps_counter_visibility = Visibility::Hidden;
    }
}

fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default()
    ));
}

fn setup_fps_counter(
    mut commands: Commands
) {
    commands.spawn((
        TextBundle{
            text: Text::from_section(
                "FPS: 0.0",
                TextStyle {
                    font: default(),
                    font_size: 20.0,
                    color: Color::WHITE,
                }
            ),
            style: Style {
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                position_type: PositionType::Absolute,
               ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
        FpsCounter
    ));
}

fn measure_fps(
    mut fps_query: Query<&mut Text, With<FpsCounter>>,
    diagnostics: Res<DiagnosticsStore>
) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed()) {
        let mut fps_text = fps_query.get_single_mut().unwrap();
        fps_text.sections[0].value = format!("FPS: {}", fps);
    }
}