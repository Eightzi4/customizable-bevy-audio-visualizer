#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use audio_visualizer::dynamic::live_input::AudioDevAndCfg;
use bevy::{
    prelude::*, window::PrimaryWindow
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::na::clamp;
use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream
};
use ringbuffer::{
    AllocRingBuffer, 
    RingBuffer
};
use std::{f64::consts::PI, sync::{
    Arc, 
    Mutex
}};
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::hann_window;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_non_send_resource::<AudioData>()
        .init_resource::<AudioVisualizerUpdateTimer>()
        //.add_systems(PreUpdate, measure_fps)// uncomment to measure fps
        .add_systems(Startup, (setup_audio_data_updater, setup_audio_visualizer, spawn_camera))
        .add_systems(Update, (tick_audio_visulaizer_update_timer, visualize_audio_frequency_symmetrical, update_audio_visualizer_scale, update_audio_visualizer_rotation))
        .run();
}

const COLUMN_COUNT: usize = 256;
const SPECTRUM_DATA_LENGTH: usize = 8192;
const RADIUS: f64 = 200.0;
const ANGLE_INCREMENT: f64 = 2.0 * PI / COLUMN_COUNT as f64;
const FREQUENCIES_COMBINATION_SMOOTHING: usize = 4;
const AV_BEAT_SCALE_SMOOTHING: f32 = 0.5;
const AV_ROTATION_SPEED: f32 = 0.005;

#[derive(Resource)]
struct AudioVisualizerUpdateTimer{
    timer: Timer
}

impl Default for AudioVisualizerUpdateTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.032, TimerMode::Repeating)
        }
    }
}

struct AudioData {
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    stream: Option<Stream>,
    latest_average_frequency_value: f32
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            latest_audio_data: Arc::new(Mutex::new(AllocRingBuffer::new(SPECTRUM_DATA_LENGTH))),
            stream: None,
            latest_average_frequency_value: 0.0
        }
    }
}


#[derive(Component)]
struct AudioVisualizerContainer;

fn spawn_camera(
    mut commands: Commands,
) {

    commands.spawn(Camera2dBundle {
        ..default()
    });
}

#[derive(Component)]
struct InnerAudioVisualizerCollumn;

#[derive(Component)]
struct OuterAudioVisualizerCollumn;

fn setup_audio_data_updater(
    mut audio_data: NonSendMut<AudioData>,
) {
    let output_device = cpal::default_host().default_output_device().unwrap();

    let preffered_cfg = output_device.default_output_config().unwrap();
    let latest_audio_data = &audio_data.latest_audio_data;

    let audio_dev_and_cfg = AudioDevAndCfg::new(
        Some(output_device.clone()),
        Some(preffered_cfg.clone().into()),
    );
    let stream = audio_visualizer::dynamic::live_input::setup_audio_input_loop(
        latest_audio_data.clone(),
        audio_dev_and_cfg,
    );

    audio_data.stream = Some(stream);
    audio_data.stream.as_ref().unwrap().play().unwrap();
}

fn setup_audio_visualizer(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>
) {
    let window = window.get_single().unwrap();
    let column_width = window.width() / COLUMN_COUNT as f32;
    let window_height = window.height();

    commands.spawn((
        NodeBundle {
            style: Style {
                height: Val::Px(window_height),
                width: Val::Px(window_height),
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                display: Display::Flex,
                ..default()
            },
            ..default()
        },
        AudioVisualizerContainer
    )
    ).with_children(|parent| {
        for i in 0..COLUMN_COUNT {
            let angle = i as f64 * ANGLE_INCREMENT;
            let x = (RADIUS * angle.cos()) as f32 + window_height / 2.0;
            let y = (RADIUS * angle.sin()) as f32 + window_height / 2.0;
    
            parent.spawn((
                NodeBundle {
                    transform: Transform {
                        rotation: Quat::from_rotation_z((angle - PI / 2.0) as f32),                        
                        ..default()
                    },
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(x),
                        top: Val::Px(y),
                        ..default()
                    },
                    ..default()
                },
                OuterAudioVisualizerCollumn
                )
            ).with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            height: Val::Px(2.0),
                            width: Val::Px(column_width),
                            position_type: PositionType::Absolute,
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::BLACK),
                        ..default()
                    },
                    InnerAudioVisualizerCollumn
                ));
            });
        }
    });
}

fn tick_audio_visulaizer_update_timer(
    time: Res<Time>,
    mut audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    audio_visualizer_update_timer.timer.tick(time.delta());
}

fn measure_fps(
    diagnostics: Res<DiagnosticsStore>
) {
    if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed()) {
        println!("FPS: {}", value);
    }
}

fn visualize_audio_spectrum(
    audio_data: NonSend<AudioData>,
    mut column_query: Query<&mut Style, With<InnerAudioVisualizerCollumn>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        
        let spectrum_data = audio_data.latest_audio_data.lock().unwrap();
        
        if spectrum_data.len() == SPECTRUM_DATA_LENGTH {
            let window = window_query.get_single().unwrap();

            let max_height = window.height();

            for (i, mut style) in column_query.iter_mut().enumerate() {
                let current_height = style.height.resolve(1.0, Vec2::new(1.0, 1.0)).unwrap();
                let target_height = spectrum_data[i] * max_height * 5.0;

                let new_height = current_height + (target_height - current_height) * 0.5;

                style.height = Val::Px(new_height.max(0.0));
            }
        } else {
            println!("Not enough data to visualize!");
        }
    }
}

fn visualize_audio_frequency(
    mut audio_data: NonSendMut<AudioData>,
    mut column_query: Query<(&mut Style, &mut BackgroundColor), With<InnerAudioVisualizerCollumn>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let spectrum_data = audio_data.latest_audio_data.lock().unwrap();
        
        if spectrum_data.len() == SPECTRUM_DATA_LENGTH {
            let window = window_query.get_single().unwrap();

            let max_height = window.height();

            let spectrum_data_clone: [f32; 8192] = spectrum_data.iter().copied().collect::<Vec<f32>>().try_into().unwrap();
            drop(spectrum_data);

            //let hann_window = hann_window(&spectrum_data_clone);

            let frequencies = samples_fft_to_spectrum(
                //&hann_window,
                &spectrum_data_clone,
                4096,
                FrequencyLimit::Range(20.0, 1300.0),
                Some(&scale_to_zero_to_one),
            ).unwrap();

            audio_data.latest_average_frequency_value = frequencies.average().val();

            let mut combined_frequencies: [f32; COLUMN_COUNT] = [0.0; COLUMN_COUNT];
            for i in 0..COLUMN_COUNT {
                for j in i * 10..(i + 1) * 10 {
                    combined_frequencies[i] += frequencies.data().get(j).unwrap().1.val();
                }
            }
            for (i, (mut style, mut background_color)) in column_query.iter_mut().enumerate() {
                let mut smooth_frequency = 0.0;
                let mut index: i32 = (i as i32 - (FREQUENCIES_COMBINATION_SMOOTHING / 2) as i32 + COLUMN_COUNT as i32) % COLUMN_COUNT as i32;
                for _ in 0..FREQUENCIES_COMBINATION_SMOOTHING {
                    smooth_frequency += combined_frequencies[index as usize];
                    index = (index + 1) % COLUMN_COUNT as i32;
                }
                combined_frequencies[i] = smooth_frequency / FREQUENCIES_COMBINATION_SMOOTHING as f32;

                let new_height = combined_frequencies[i] / 10.0 * max_height / frequencies.max().1.val();
                
                style.height = Val::Px(clamp(new_height, 2.0, 200.0));

                if combined_frequencies[i] / 10.0 > frequencies.average().val() * 2.0 {
                    background_color.0 = Color::RED;
                } else {
                    background_color.0 = Color::BLACK;
                }
            }
        } else {
            println!("Not enough data to visualize!");
        }
    }
}

fn visualize_audio_frequency_symmetrical(
    mut audio_data: NonSendMut<AudioData>,
    mut column_query: Query<(&mut Style, &mut BackgroundColor), With<InnerAudioVisualizerCollumn>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let spectrum_data = audio_data.latest_audio_data.lock().unwrap();
        
        if spectrum_data.len() == SPECTRUM_DATA_LENGTH {
            let window = window_query.get_single().unwrap();

            let max_height = window.height();

            let spectrum_data_clone: [f32; 8192] = spectrum_data.iter().copied().collect::<Vec<f32>>().try_into().unwrap();
            drop(spectrum_data);

            //let hann_window = hann_window(&spectrum_data_clone);

            let frequencies = samples_fft_to_spectrum(
                //&hann_window,
                &spectrum_data_clone,
                4096,
                FrequencyLimit::Range(20.0, 1300.0),
                Some(&scale_to_zero_to_one),
            ).unwrap();

            audio_data.latest_average_frequency_value = frequencies.average().val();

            let mut combined_frequencies: [f32; COLUMN_COUNT / 2] = [0.0; COLUMN_COUNT / 2];
            for i in 0..COLUMN_COUNT / 2 {
                for j in i * 10..(i + 1) * 10 {
                    combined_frequencies[i] += frequencies.data().get(j).unwrap().1.val();
                }
            }
            for (i, (mut style, mut background_color)) in column_query.iter_mut().enumerate() {
                let clamped_index = i % (COLUMN_COUNT / 2);
            
                let new_height;
                if i < COLUMN_COUNT / 2 {
                    let mut smooth_frequency = 0.0;
                    let mut index: i32 = (clamped_index as i32 - (FREQUENCIES_COMBINATION_SMOOTHING / 2) as i32 + COLUMN_COUNT as i32 / 2) % (COLUMN_COUNT / 2) as i32;
                    for _ in 0..FREQUENCIES_COMBINATION_SMOOTHING {
                        smooth_frequency += combined_frequencies[index as usize];
                        index = (index + 1) % (COLUMN_COUNT / 2) as i32;
                    }
                    combined_frequencies[clamped_index] = smooth_frequency / FREQUENCIES_COMBINATION_SMOOTHING as f32;
                    new_height = combined_frequencies[clamped_index] / 10.0 * max_height / frequencies.max().1.val();
                } else {
                    let opposite_index = (i - COLUMN_COUNT / 2) % (COLUMN_COUNT / 2);
                    new_height = combined_frequencies[opposite_index] / 10.0 * max_height / frequencies.max().1.val();
                }
            
                style.height = Val::Px(clamp(new_height, 2.0, 200.0));
            
                if combined_frequencies[clamped_index] / 10.0 > frequencies.average().val() * 2.0 {
                    background_color.0 = Color::RED;
                } else {
                    background_color.0 = Color::BLACK;
                }
            }
        } else {
            println!("Not enough data to visualize!");
        }
    }
}

fn update_audio_visualizer_scale(
    audio_data: NonSend<AudioData>,
    mut audio_visulizer_container_query: Query<&mut Style, With<OuterAudioVisualizerCollumn>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let window_height = window_query.single().height();
        let radius_scaler = audio_data.latest_average_frequency_value as f64 * 1000.0;
        for (i, mut style) in audio_visulizer_container_query.iter_mut().enumerate() {
            let current_scale_x = style.left.resolve(1.0, Vec2::new(1.0, 1.0)).unwrap();
            let current_scale_y = style.top.resolve(1.0, Vec2::new(1.0, 1.0)).unwrap();
            let angle = i as f64 * ANGLE_INCREMENT;
            let x = ((RADIUS + radius_scaler) * angle.cos()) as f32 + window_height / 2.0;
            let y = ((RADIUS + radius_scaler) * angle.sin()) as f32 + window_height / 2.0;

            style.left = Val::Px((x - current_scale_x) * AV_BEAT_SCALE_SMOOTHING + current_scale_x);
            style.top = Val::Px((y - current_scale_y) * AV_BEAT_SCALE_SMOOTHING + current_scale_y);
        }
    }
}

fn update_audio_visualizer_rotation(
    //audio_data: NonSend<AudioData>,
    mut audio_visulizer_container_query: Query<&mut Transform, With<AudioVisualizerContainer>>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        //let latest_average_frequency_value = audio_data.latest_average_frequency_value;
        let mut audio_visualizer_container = audio_visulizer_container_query.get_single_mut().unwrap();
        audio_visualizer_container.rotation *= Quat::from_rotation_z(AV_ROTATION_SPEED);
    }
}