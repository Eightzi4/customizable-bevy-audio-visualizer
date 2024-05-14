use bevy::prelude::*;

mod systems;

use systems::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, settings_ui);
    }
}