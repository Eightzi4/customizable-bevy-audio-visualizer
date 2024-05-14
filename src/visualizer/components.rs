use std::f32::consts::PI;
use bevy::prelude::*;

#[derive(Component)]
pub struct AudioVisualizerContainer;

#[derive(Component)]
pub struct AudioVisualizerCollumn;

#[derive(Resource)]
pub struct AudioVisualizerSettings {
    pub spectrum_data_length: usize,
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
    pub scale_treshold: f32,
    pub normal_color_material_handle: Option<Handle<ColorMaterial>>,
    pub normal_color_hdr_multiplier: f32,
    pub highlight_color_material_handle: Option<Handle<ColorMaterial>>,
    pub highlight_color_hdr_multiplier: f32
}

impl Default for AudioVisualizerSettings {
    fn default() -> Self {
        Self {
            spectrum_data_length: 8192,
            column_count: 256,
            column_count_power_of_two: 8,
            section_count: 1,
            radius: 200.0,
            rotation_speed: 0.005,
            angle_increment: 2.0 * PI / 256.0,
            column_width: 2.5,
            max_height: 500.0,
            smoothing_range: 4,
            scale_strenght: 1000.0,
            scale_treshold: 2.0,
            normal_color_material_handle: None,
            normal_color_hdr_multiplier: 1.0,
            highlight_color_material_handle: None,
            highlight_color_hdr_multiplier: 1.0
        }
    }
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