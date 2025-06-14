use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};

use crate::{
    player::Player,
    render_layer::{PORTAL_LAYER, WORLD_LAYER},
    SKY_COLOUR,
};

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PortalEntranceMaterial>::default())
            .add_systems(
                PostUpdate,
                align_portal_cameras.before(TransformSystem::TransformPropagate),
            )
            .add_systems(Startup, spawn_portals)
            .add_observer(setup_portal_camera);
    }
}

#[derive(Component)]
pub struct PortalEntrance {
    exit: Option<Entity>,
}

#[derive(Component)]
#[require(Transform)]
pub struct PortalCamera {
    entrance: Entity,
    exit: Entity,
}

fn spawn_portals(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let portal_mesh_dimensions = Vec3::new(4.0, 4.0, 0.0);
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
        });
    commands
        .entity(portal_b_id)
        .insert(PortalEntrance {
            exit: Some(portal_a_id),
        });
}

fn setup_portal_camera(
    trigger: Trigger<OnAdd, PortalEntrance>,
    mut commands: Commands,
    q_portal: Query<&PortalEntrance>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
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
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: 70_f32.to_radians(),
            ..default()
        }),
        Mesh3d(meshes.add(Cuboid::from_length(0.25))),
        MeshMaterial3d(
            standard_materials.add(StandardMaterial::from_color(Color::linear_rgb(
                1.0, 0.0, 0.0,
            ))),
        ),
        RenderLayers::layer(WORLD_LAYER),
    ));

    // Add texture to entrance portal
    let portal_entrance_material_handle =
        portal_materials.add(PortalEntranceMaterial { image_handle });
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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct PortalEntranceMaterial {
    #[texture(0)]
    #[sampler(1)]
    image_handle: Handle<Image>,
}

impl Material for PortalEntranceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/portal_entrance.wgsl".into()
    }
}
