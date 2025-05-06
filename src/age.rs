use bevy::prelude::*;

pub struct AgePlugin;

impl Plugin for AgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_age, despawn_at_end_of_lifespan));
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

#[derive(Component, Debug)]
#[require(Age)]
pub struct Lifespan {
    pub seconds: f32,
}

fn despawn_at_end_of_lifespan(mut commands: Commands, q: Query<(Entity, &Age, &Lifespan)>) {
    q.iter()
        .filter_map(|(entity, age, lifespan)| {
            if age.seconds < lifespan.seconds {
                Some(entity)
            } else {
                None
            }
        })
        .for_each(|entity| {
            commands
                .entity(entity)
                .try_despawn_recursive()
        });
}
