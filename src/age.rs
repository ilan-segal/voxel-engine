use bevy::prelude::*;

pub struct AgePlugin;

impl Plugin for AgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_age);
    }
}

#[derive(Component, Default, Debug)]
pub struct Age {
    pub seconds: f32,
}

fn update_age(mut q: Query<&mut Age>, time: Res<Time>) {
    let delta_seconds = time.delta_secs();
    for mut age in q.iter_mut() {
        age.seconds += delta_seconds;
    }
}
