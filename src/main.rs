use voxels::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<crate::shader::CustomMaterial>::default(),
        ))
        .add_systems(Startup, (
            crate::chunk::spawn_world,
            crate::camera::spawn_camera,
        ))
        .add_systems(Update, crate::camera::move_cam)
        .run();
}