use bevy::app::Plugin;

pub mod material;
pub mod mesh;
pub mod texture;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((mesh::MeshPlugin, texture::TexturePlugin));
    }
}
