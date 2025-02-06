use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    utils::hashbrown::HashMap,
};
use itertools::Itertools;

pub struct TerrainMeshPlugin;

impl Plugin for TerrainMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut map = HashMap::new();
    let ao_factor_range = 0..=3;
    for (((a, b), c), d) in ao_factor_range
        .clone()
        .cartesian_product(ao_factor_range.clone())
        .cartesian_product(ao_factor_range.clone())
        .cartesian_product(ao_factor_range.clone())
    {
        let ao_factors = [a, b, c, d];
        let spec = SquareMeshSpec { ao_factors };
        let mesh_handle = meshes.add(Mesh::from(spec.clone()));
        map.insert(spec, mesh_handle);
    }
    commands.insert_resource(PrecomputedTerrainMeshes { map });
}

#[derive(Resource, Default)]
pub struct PrecomputedTerrainMeshes {
    pub map: HashMap<SquareMeshSpec, Handle<Mesh>>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct SquareMeshSpec {
    pub ao_factors: [u8; 4],
}

impl From<SquareMeshSpec> for Mesh {
    fn from(spec: SquareMeshSpec) -> Self {
        /*
        3---2
        |b /|
        | / |
        |/ a|
        0---1
        */
        let indices = vec![0, 1, 2, 0, 2, 3];
        let vertices = vec![
            [-0.5, 0., 0.5],
            [0.5, 0., 0.5],
            [0.5, 0., -0.5],
            [-0.5, 0., -0.5],
        ];
        let normals = vec![[0., 1., 0.]; 4];
        let uv = vec![[0., 1.], [1., 1.], [1., 0.], [0., 0.]];
        let colours = spec
            .ao_factors
            .iter()
            .map(move |factor| {
                let lum = 0.6_f32.powi((*factor).into());
                return Color::WHITE.with_luminance(lum);
            })
            .map(|c| c.to_linear().to_f32_array())
            .collect::<Vec<_>>();
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U16(indices));
        return mesh;
    }
}
