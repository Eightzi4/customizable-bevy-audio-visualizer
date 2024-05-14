use bevy::prelude::*;
use audio_visualizer::dynamic::live_input::AudioDevAndCfg;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use super::components::*;

pub fn setup_audio_data_updater(
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