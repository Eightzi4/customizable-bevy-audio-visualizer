use bevy::{prelude::*, window::WindowResized};
use core::f32::consts::PI;

pub mod components;
mod systems;

use systems::*;
use components::*;

pub struct AudioVisualizerPlugin;

impl Plugin for AudioVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<VisualilzerType>()
        .init_resource::<AudioVisualizerUpdateTimer>()
        .init_resource::<AudioVisualizerSettings>()
        .add_event::<AudioVisualizerRestructureEvent>()
        .add_systems(Startup, setup_audio_visualizer)
        .add_systems(Update, (tick_audio_visualizer_update_timer, update_audio_visualizer_rotation, update_audio_visualizer_scale, update_color_transition))
        .add_systems(Update, visualize_audio_spectrum.run_if(in_state(VisualilzerType::SpectrumVisualizer)))
        .add_systems(Update, visualize_audio_frequency.run_if(in_state(VisualilzerType::FrequencyVisualizer)))
        .add_systems(Update, restructure_audio_visualizer.run_if(on_event::<AudioVisualizerRestructureEvent>()))
        .add_systems(Update, center_audio_visualizer.run_if(on_event::<WindowResized>()));
    }
}