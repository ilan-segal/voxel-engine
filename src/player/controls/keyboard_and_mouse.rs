use std::f32::consts::PI;

use bevy::{
    ecs::query::QueryData,
    input::{
        common_conditions::input_just_pressed,
        keyboard::KeyboardInput,
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    },
    prelude::*,
    utils::{Entry, HashMap},
};

use super::{target_velocity::TargetVelocity, Sprinting};
use crate::{
    block::Block,
    item::{DroppedItemBundle, Item, ItemBundle, Quantity, DROPPED_ITEM_SCALE},
    physics::{aabb::Aabb, collision::Collidable, friction::Friction},
    player::{
        block_target::{TargetedBlock, TargetedSpace},
        inventory::{HotbarSelection, Inventory},
        mode::PlayerMode,
        Jumping, Player, Sneaking,
    },
    state::GameState,
    world::block_update::SetBlockEvent,
};

pub struct KeyboardMousePlugin;

impl Plugin for KeyboardMousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_input_tracker,
                track_new_press,
                age_presses,
                rotate_camera_with_mouse,
                process_keyboard_inputs,
                delete_targeted_block.run_if(input_just_pressed(MouseButton::Left)),
                place_block.run_if(input_just_pressed(MouseButton::Right)),
                drop_item.run_if(input_just_pressed(KeyCode::KeyQ)),
                change_hotbar_selection_from_keys,
                change_hotbar_selection_from_scrollbar,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .observe(toggle_player_mode);
    }
}

fn rotate_camera_with_mouse(
    mut mouse_events: EventReader<MouseMotion>,
    mut q_camera: Query<&mut Transform, (With<Player>, With<Camera3d>)>,
) {
    let mut transform = q_camera
        .get_single_mut()
        .ok()
        .expect("There should be exactly one player camera");
    const CAMERA_MOUSE_SENSITIVITY_X: f32 = 0.004;
    const CAMERA_MOUSE_SENSITIVITY_Y: f32 = 0.0025;
    for MouseMotion { delta } in mouse_events.read() {
        transform.rotate_axis(Dir3::NEG_Y, delta.x * CAMERA_MOUSE_SENSITIVITY_X);
        let (yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        pitch = (pitch - delta.y * CAMERA_MOUSE_SENSITIVITY_Y).clamp(-PI * 0.5, PI * 0.5);
        transform.rotation = Quat::from_euler(
            // YXZ order corresponds to the common
            // "yaw"/"pitch"/"roll" convention
            EulerRot::YXZ,
            yaw,
            pitch,
            0.0,
        );
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct KeyboardInputQuery {
    target_v: &'static mut TargetVelocity,
    t: &'static Transform,
    is_sprinting: &'static mut Sprinting,
    is_jumping: &'static mut Jumping,
    is_sneaking: &'static mut Sneaking,
}

fn process_keyboard_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_velocity: Query<KeyboardInputQuery, With<Player>>,
) {
    let Ok(mut object) = q_velocity.get_single_mut() else {
        return;
    };
    object.target_v.0 = Vec3::ZERO;
    let mut v_horizontal = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        v_horizontal.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        v_horizontal.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        v_horizontal.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        v_horizontal.x += 1.0;
    }
    if v_horizontal != Vec3::ZERO {
        let (yaw, _, _) = object.t.rotation.to_euler(EulerRot::YXZ);
        v_horizontal = (Quat::from_rotation_y(yaw) * v_horizontal).normalize();
        object.target_v.0 += v_horizontal;
    }

    object.is_sprinting.0 = keys.pressed(KeyCode::ControlLeft);
    object.is_jumping.0 = keys.pressed(KeyCode::Space);
    object.is_sneaking.0 = keys.pressed(KeyCode::ShiftLeft);
}

fn delete_targeted_block(
    targeted_block: Res<TargetedBlock>,
    mut set_block_events: EventWriter<SetBlockEvent>,
) {
    if let Some(pos) = targeted_block.0 {
        set_block_events.send(SetBlockEvent {
            block: Block::Air,
            world_pos: pos.to_array(),
        });
    }
}

fn place_block(
    targeted_space: Res<TargetedSpace>,
    mut q_inventory: Query<(&HotbarSelection, &mut Inventory)>,
    mut set_block_events: EventWriter<SetBlockEvent>,
) {
    let Some(space_pos) = targeted_space.0 else {
        return;
    };
    for (selection, mut inventory) in q_inventory.iter_mut() {
        let index = selection.index as usize;
        let Some(Some(ref mut item)) = inventory.hotbar.get_mut(index) else {
            continue;
        };
        let Item::Block(block) = item.item;
        item.quantity.decrease(1);
        if item.quantity == Quantity::Finite(0) {
            inventory.hotbar[index] = None;
        }
        set_block_events.send(SetBlockEvent {
            block,
            world_pos: space_pos.to_array(),
        });
    }
}

#[derive(Event, Debug)]
pub struct DoubleTap(KeyCode);

#[derive(Component, Default)]
pub struct SecondsSinceLastPress(HashMap<KeyCode, f32>);

fn add_input_tracker(
    q: Query<Entity, (With<Player>, Without<SecondsSinceLastPress>)>,
    mut commands: Commands,
) {
    for entity in q.iter() {
        commands
            .entity(entity)
            .insert(SecondsSinceLastPress::default());
    }
}

fn track_new_press(
    mut q_tracker: Query<(Entity, &mut SecondsSinceLastPress)>,
    inputs: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for key_code in inputs.get_just_pressed().copied() {
        for (entity, mut tracker) in q_tracker.iter_mut() {
            let entry = tracker.0.entry(key_code);
            if let Entry::Occupied(..) = entry {
                let event = DoubleTap(key_code);
                commands.trigger_targets(event, entity);
            }
            entry.insert(0.0);
        }
    }
}

const DOUBLE_TAP_DURATION_SECONDS: f32 = 0.5;

fn age_presses(mut q_tracker: Query<&mut SecondsSinceLastPress>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut q_tracker in q_tracker.iter_mut() {
        q_tracker.0.retain(|_, duration_s| {
            *duration_s += dt;
            return *duration_s <= DOUBLE_TAP_DURATION_SECONDS;
        });
    }
}

const NO_CLIP_TOGGLE: KeyCode = KeyCode::KeyZ;

fn toggle_player_mode(trigger: Trigger<DoubleTap>, mut q_mode: Query<&mut PlayerMode>) {
    let event = trigger.event();
    if event.0 != NO_CLIP_TOGGLE {
        return;
    }
    let entity = trigger.entity();
    let Ok(mut mode) = q_mode.get_mut(entity) else {
        return;
    };
    *mode = match *mode {
        PlayerMode::Survival => PlayerMode::NoClip,
        PlayerMode::NoClip => PlayerMode::Survival,
    };
    info!("Switched to {:?}", *mode);
}

fn change_hotbar_selection_from_keys(
    mut q_hotbar_selection: Query<&mut HotbarSelection>,
    mut key_presses: EventReader<KeyboardInput>,
) {
    let Ok(mut selection) = q_hotbar_selection.get_single_mut() else {
        warn!("Failed to find editable selection");
        return;
    };
    for key in key_presses.read() {
        let maybe_index = match key.key_code {
            KeyCode::Digit1 => Some(0),
            KeyCode::Digit2 => Some(1),
            KeyCode::Digit3 => Some(2),
            KeyCode::Digit4 => Some(3),
            KeyCode::Digit5 => Some(4),
            KeyCode::Digit6 => Some(5),
            KeyCode::Digit7 => Some(6),
            KeyCode::Digit8 => Some(7),
            KeyCode::Digit9 => Some(8),
            KeyCode::Digit0 => Some(9),
            _ => None,
        };
        if let Some(index) = maybe_index {
            selection.index = index;
        }
    }
}

fn change_hotbar_selection_from_scrollbar(
    mut q_hotbar_selection: Query<&mut HotbarSelection>,
    mut key_presses: EventReader<MouseWheel>,
) {
    let Ok(mut selection) = q_hotbar_selection.get_single_mut() else {
        warn!("Failed to find editable selection");
        return;
    };
    for wheel_event in key_presses.read() {
        let delta = match wheel_event.unit {
            MouseScrollUnit::Line => wheel_event.y.signum() as i8,
            _ => 0,
        };
        match delta {
            0 => {}
            1 => selection.increase(),
            -1 => selection.decrease(),
            _ => unreachable!(),
        }
    }
}

fn drop_item(
    mut q_inventory: Query<(&HotbarSelection, &mut Inventory, &GlobalTransform)>,
    mut commands: Commands,
) {
    for (selection, mut inventory, global_transform) in q_inventory.iter_mut() {
        let index = selection.index as usize;
        let Some(Some(ref mut item)) = inventory.hotbar.get_mut(index) else {
            continue;
        };
        item.quantity.decrease(1);
        let Item::Block(block) = item.item;
        let player_transform = global_transform.compute_transform();
        commands.spawn(DroppedItemBundle {
            item: ItemBundle {
                item: Item::Block(block),
                quantity: item.quantity,
            },
            spatial: SpatialBundle::from_transform(Transform {
                translation: player_transform.translation,
                scale: Vec3::ONE * DROPPED_ITEM_SCALE,
                ..default()
            }),
            chunk_position: default(),
            gravity: default(),
            velocity: (player_transform.rotation * Vec3::NEG_Z * 5.0).into(),
            aabb: Aabb::cube(1.0),
            collidable: Collidable,
            friction: Friction { coefficient: 0.1 },
        });
    }
}
