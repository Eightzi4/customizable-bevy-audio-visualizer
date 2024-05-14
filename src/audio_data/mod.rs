use bevy::prelude::*;

pub mod components;
mod systems;

use systems::*;
use components::*;

pub struct AudioDataPlugin;

impl Plugin for AudioDataPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_non_send_resource::<AudioData>()
        .add_systems(Startup, setup_audio_data_updater);
    }
}

pub const SPECTRUM_DATA_LENGTH: usize = 8192;