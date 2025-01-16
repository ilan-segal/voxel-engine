use bevy::prelude::*;

use crate::player::{
    health::{Health, MaxHealth},
    Player,
};

use super::Ui;

pub struct HealthUiPlugin;

impl Plugin for HealthUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(super::setup))
            .add_systems(Update, update_health_display);
    }
}

#[derive(Resource)]
struct SpriteHandles {
    full_heart: UiImage,
    half_heart: UiImage,
    container: UiImage,
}

#[derive(Component)]
pub struct HealthDisplayRoot;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = SpriteHandles {
        full_heart: UiImage::new(asset_server.load("ui/hud/heart/full.png")),
        half_heart: UiImage::new(asset_server.load("ui/hud/heart/half.png")),
        container: UiImage::new(asset_server.load("ui/hud/heart/container.png")),
    };
    commands.insert_resource(handles);
}

const HEART_SPRITE_SIZE: f32 = 25.0;

fn update_health_display(
    mut commands: Commands,
    q_health: Query<(&Health, Option<&MaxHealth>), (With<Player>, Changed<Health>)>,
    q_display_root: Query<Entity, With<HealthDisplayRoot>>,
    sprites: Res<SpriteHandles>,
) {
    let Ok((Health(health), max_health)) = q_health.get_single() else {
        return;
    };
    let Ok(display_root_entity) = q_display_root.get_single() else {
        warn!("Could not find health display root");
        return;
    };
    let num_containers = match max_health {
        Some(MaxHealth(n)) => n.div_ceil(2),
        None => health.div_ceil(2),
    };
    commands
        .entity(display_root_entity)
        .despawn_descendants()
        .with_children(|builder| {
            for i in 1..=num_containers {
                let width = Val::Px(HEART_SPRITE_SIZE);
                let height = Val::Px(HEART_SPRITE_SIZE);
                builder
                    .spawn((
                        Ui,
                        NodeBundle {
                            style: Style {
                                width,
                                height,
                                ..default()
                            },
                            ..default()
                        },
                        sprites.container.clone(),
                    ))
                    .with_children(|builder_2| {
                        let heart_sprite = if health >= &(i * 2) {
                            Some(sprites.full_heart.clone())
                        } else if health + 1 >= i * 2 {
                            Some(sprites.half_heart.clone())
                        } else {
                            None
                        };
                        if let Some(sprite) = heart_sprite {
                            builder_2.spawn((
                                Ui,
                                NodeBundle {
                                    style: Style {
                                        width,
                                        height,
                                        ..default()
                                    },
                                    ..default()
                                },
                                sprite,
                            ));
                        }
                    });
            }
        });
}
