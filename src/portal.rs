use bevy::{
    asset::RenderAssetUsages,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};

use crate::{
    physics::PhysicsSystemSet,
    player::Player,
    render_layer::{PORTAL_LAYER, WORLD_LAYER},
    SKY_COLOUR,
};

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PortalEntranceMaterial>::default())
            .add_systems(Startup, spawn_portals)
            .add_systems(PreUpdate, add_prev_position_component)
            .add_systems(
                Update,
                (
                    update_prev_position.before(PhysicsSystemSet::Act),
                    move_through_portals.after(PhysicsSystemSet::React),
                ),
            )
            .add_systems(
                PostUpdate,
                align_portal_cameras.before(TransformSystem::TransformPropagate),
            )
            .add_observer(setup_portal_camera);
    }
}

#[derive(Component)]
pub struct PortalEntrance {
    exit: Option<Entity>,
    size: Vec2,
}

#[derive(Component)]
#[require(Transform)]
pub struct PortalCamera {
    entrance: Entity,
    exit: Entity,
}

fn spawn_portals(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let size = Vec2::new(4.0, 4.0);
    let portal_mesh_dimensions = size.extend(0.0);
    let rectangle = meshes.add(Cuboid::from_size(portal_mesh_dimensions));
    let portal_a_id = commands
        .spawn((
            Mesh3d(rectangle.clone()),
            Transform::from_xyz(-3.0, 1.0 + portal_mesh_dimensions.y * 0.5, 5.5),
            RenderLayers::layer(WORLD_LAYER),
        ))
        .id();
    let portal_b_id = commands
        .spawn((
            Mesh3d(rectangle.clone()),
            Transform::from_xyz(-46.0, 22.0 + portal_mesh_dimensions.y * 0.5, 20.5),
            RenderLayers::layer(WORLD_LAYER),
        ))
        .id();
    commands
        .entity(portal_a_id)
        .insert(PortalEntrance {
            exit: Some(portal_b_id),
            size,
        });
    commands
        .entity(portal_b_id)
        .insert(PortalEntrance {
            exit: Some(portal_a_id),
            size,
        });
}

fn setup_portal_camera(
    trigger: Trigger<OnAdd, PortalEntrance>,
    mut commands: Commands,
    q_portal: Query<&PortalEntrance>,
    mut portal_materials: ResMut<Assets<PortalEntranceMaterial>>,
    mut images: ResMut<Assets<Image>>,
    window: Single<&Window>,
) {
    let entrance = trigger.target();
    let Some(exit) = q_portal
        .get(entrance)
        .ok()
        .and_then(|portal| portal.exit)
    else {
        warn!("Could not find exit!");
        return;
    };

    // Set up portal texture, which the camera will render to
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        ..default()
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);

    // Portal Camera
    commands.spawn((
        PortalCamera { entrance, exit },
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            clear_color: SKY_COLOUR.into(),
            order: -1,
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: 70_f32.to_radians(),
            near: 0.0001,
            ..default()
        }),
        RenderLayers::layer(WORLD_LAYER),
    ));

    // Add texture to entrance portal
    let portal_entrance_material_handle = portal_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            double_sided: true,
            cull_mode: None,
            ..default()
        },
        extension: PortalMaterialExtension { image_handle },
    });
    commands.entity(entrance).insert((
        MeshMaterial3d(portal_entrance_material_handle),
        RenderLayers::layer(PORTAL_LAYER),
    ));
}

fn align_portal_cameras(
    q_player_camera_transform: Single<
        &Transform,
        (
            With<Camera3d>,
            Without<PortalCamera>,
            With<Player>,
            Without<PortalEntrance>,
        ),
    >,
    q_portals: Query<&Transform, With<PortalEntrance>>,
    mut q_portal_cameras: Query<
        (&mut Transform, &PortalCamera),
        (With<PortalCamera>, Without<PortalEntrance>),
    >,
) {
    let eye_affine = q_player_camera_transform.compute_affine();
    for (mut portal_camera_transform, camera_data) in q_portal_cameras.iter_mut() {
        let Ok(entrance_transform) = q_portals.get(camera_data.entrance) else {
            continue;
        };
        let Ok(exit_transform) = q_portals.get(camera_data.exit) else {
            continue;
        };
        let entrance_affine = entrance_transform.compute_affine();
        let exit_affine = exit_transform.compute_affine();
        let camera_affine = exit_affine * entrance_affine.inverse() * eye_affine;
        *portal_camera_transform = Transform::from_matrix(camera_affine.into());
    }
}

type PortalEntranceMaterial = ExtendedMaterial<StandardMaterial, PortalMaterialExtension>;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct PortalMaterialExtension {
    #[texture(100)]
    #[sampler(101)]
    image_handle: Handle<Image>,
}

impl MaterialExtension for PortalMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/portal_entrance.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        Self::fragment_shader()
    }
}

#[derive(Component, Default)]
struct PreviousPosition(Vec3);

fn add_prev_position_component(
    mut commands: Commands,
    q_no_prev_position: Query<(Entity, &Transform), Without<PreviousPosition>>,
) {
    for (entity, transform) in q_no_prev_position.iter() {
        commands
            .entity(entity)
            .insert(PreviousPosition(transform.translation));
    }
}

fn update_prev_position(mut q_prev_position: Query<(&Transform, &mut PreviousPosition)>) {
    for (transform, mut prev_position) in q_prev_position.iter_mut() {
        prev_position.0 = transform.translation;
    }
}

// Don't teleport child entities, since they'll just move with their parent
fn move_through_portals(
    mut q_teleportable: Query<
        (&PreviousPosition, &mut Transform),
        (Without<ChildOf>, Without<PortalEntrance>),
    >,
    q_portal: Query<(&Transform, &PortalEntrance)>,
) {
    for (prev_position, mut transform) in q_teleportable.iter_mut() {
        for (portal_entrance_transform, portal_entrance) in q_portal.iter() {
            if !portal_is_crossed(
                portal_entrance,
                portal_entrance_transform,
                &transform.translation,
                &prev_position.0,
            ) {
                continue;
            }
            let Some((exit_transform, _)) = portal_entrance
                .exit
                .and_then(|e| q_portal.get(e).ok())
            else {
                warn!("Could not find exit for teleportation!");
                continue;
            };
            let player_affine = transform.compute_affine();
            let entrance_affine = portal_entrance_transform.compute_affine();
            let exit_affine = exit_transform.compute_affine();
            let teleported_affine = exit_affine * entrance_affine.inverse() * player_affine;
            *transform = Transform::from_matrix(teleported_affine.into());
            break;
        }
    }
}

fn portal_is_crossed(
    portal_entrance: &PortalEntrance,
    portal_entrance_transform: &Transform,
    entity_position: &Vec3,
    entity_prev_position: &Vec3,
) -> bool {
    let x0 = entity_prev_position;
    let x1 = entity_position;
    if x0 == x1 {
        return false;
    }
    let d = x1 - x0;
    let p0 = portal_entrance_transform.translation;
    let p1 =
        portal_entrance_transform.transform_point(Vec3::new(portal_entrance.size.x * 0.5, 0., 0.));
    let p2 =
        portal_entrance_transform.transform_point(Vec3::new(0., portal_entrance.size.y * 0.5, 0.));
    let pw = p1 - p0;
    let ph = p2 - p0;
    let n = pw.cross(ph);
    let t = (p0 - x0).dot(n) / d.dot(n);
    let c = x0 + t * d;
    let u = pw.normalize().dot(c - p0) / pw.length();
    let v = ph.normalize().dot(c - p0) / ph.length();
    return is_in_range(t, 0., 1.) && is_in_range(u, -1., 1.) && is_in_range(v, -1., 1.);
}

fn is_in_range<T: PartialOrd>(value: T, min: T, max: T) -> bool {
    min <= value && value <= max
}
