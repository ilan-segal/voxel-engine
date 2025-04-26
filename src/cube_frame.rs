use bevy::prelude::*;
use bevy_polyline::prelude::*;

pub struct FramePlugin;

impl Plugin for FramePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.in_set(CubeFrameSetup));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CubeFrameSetup;

#[derive(Resource)]
pub struct CubeFrameMeshHandle(pub PolylineHandle);

fn setup(mut commands: Commands, mut polylines: ResMut<Assets<Polyline>>) {
    /*
    A___________B
    |`          :\
    | `         : \
    Y  `        :  \
    |   C-----------D
    |   :       :   :
    E__ : _X____F   :
    `   :        \  :
     Z  :         \ :
      ` :          \:
       `G___________H
    */
    const A: Vec3 = Vec3::new(0., 1., 0.);
    const B: Vec3 = Vec3::new(1., 1., 0.);
    const C: Vec3 = Vec3::new(0., 1., 1.);
    const D: Vec3 = Vec3::new(1., 1., 1.);
    const E: Vec3 = Vec3::new(0., 0., 0.);
    const F: Vec3 = Vec3::new(1., 0., 0.);
    const G: Vec3 = Vec3::new(0., 0., 1.);
    const H: Vec3 = Vec3::new(1., 0., 1.);
    let cube_frame = polylines.add(Polyline {
        vertices: [A, B, D, C, A, E, F, B, F, H, D, H, G, C, G, E]
            .iter()
            .map(|v| *v - Vec3::ONE * 0.5)
            .collect::<_>(),
    });
    commands.insert_resource(CubeFrameMeshHandle(PolylineHandle(cube_frame)));
}
