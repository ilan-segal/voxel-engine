use bevy::prelude::*;

mod keyboard_and_mouse;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(keyboard_and_mouse::KeyboardMousePlugin);
    }
}
