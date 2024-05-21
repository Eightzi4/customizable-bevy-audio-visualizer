use std::f32::consts::PI;
use bevy::prelude::*;

#[derive(Component)]
pub struct AudioVisualizerContainer;

#[derive(Component)]
pub struct AudioVisualizerColumn;

#[derive(Resource)]
pub struct AudioVisualizerSettings {
    pub spectrum_data_length: usize,
    pub lower_frequency_limit: f32,
    pub upper_frequency_limit: f32,
    pub sampling_rate: u32,
    pub window_function: WindowFunction,
    pub column_count: usize,
    pub column_count_power_of_two: usize,
    pub section_count: usize,
    pub radius: f32,
    pub rotation_speed: f32,
    pub angle_increment: f32,
    pub column_width: f32,
    pub max_height: f32,
    pub smoothing_range: usize,
    pub scale_strenght: f32,
    pub scale_threshold: f32,
    pub normal_color_material_handle: Option<Handle<ColorMaterial>>,
    pub normal_color_transition_enabled: bool,
    pub normal_color_transition_speed: f32,
    pub normal_primary_color: Color,
    pub normal_secondary_color: Color,
    pub normal_primary_color_hdr_multiplier: f32,
    pub normal_secondary_color_hdr_multiplier: f32,
    pub normal_color_transition_progress: f32,
    pub highlight_color_material_handle: Option<Handle<ColorMaterial>>,
    pub highlight_color_transition_enabled: bool,
    pub highlight_color_transition_speed: f32,
    pub highlight_primary_color: Color,
    pub highlight_secondary_color: Color,
    pub highlight_primary_color_hdr_multiplier: f32,
    pub highlight_secondary_color_hdr_multiplier: f32,
    pub highlight_color_transition_progress: f32,
}

impl Default for AudioVisualizerSettings {
    fn default() -> Self {
        Self {
            spectrum_data_length: 8192,
            lower_frequency_limit: 20.0,
            upper_frequency_limit: 1555.5,
            sampling_rate: 4096,
            window_function: WindowFunction::None,
            column_count: 256,
            column_count_power_of_two: 8,
            section_count: 1,
            radius: 200.0,
            rotation_speed: 0.005,
            angle_increment: 2.0 * PI / 256.0,
            column_width: 2.5,
            max_height: 400.0,
            smoothing_range: 4,
            scale_strenght: 1000.0,
            scale_threshold: 2.0,
            normal_color_material_handle: None,
            normal_color_transition_enabled: false,
            normal_color_transition_speed: 0.005,
            normal_primary_color: Color::WHITE,
            normal_secondary_color: Color::WHITE,
            normal_primary_color_hdr_multiplier: 1.0,
            normal_secondary_color_hdr_multiplier: 1.0,
            normal_color_transition_progress: 0.0,
            highlight_color_material_handle: None,
            highlight_color_transition_enabled: false,
            highlight_color_transition_speed: 0.005,
            highlight_primary_color: Color::RED,
            highlight_secondary_color: Color::RED,
            highlight_primary_color_hdr_multiplier: 1.0,
            highlight_secondary_color_hdr_multiplier: 1.0,
            highlight_color_transition_progress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum WindowFunction {
    None,
    Hann,
    Hamming
}

#[derive(Resource)]
pub struct AudioVisualizerUpdateTimer{
    pub timer: Timer
}

impl Default for AudioVisualizerUpdateTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.032, TimerMode::Repeating)
        }
    }
}

#[derive(Event)]
pub struct AudioVisualizerRestructureEvent;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum VisualilzerType {
    #[default]
    FrequencyVisualizer,
    SpectrumVisualizer
}