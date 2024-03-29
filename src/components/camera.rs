use bevy::prelude::*;

pub fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(50.0, 50., 50.0).looking_at(Vec3::new(0., 50., 0.), Vec3::Y),
        ..default()
    });
}

pub fn move_cam(
    time: Res<Time>,
    key: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let speed = 10.0;

    for mut transform in query.iter_mut() {
        let forward = Vec3::new(
            transform.forward().x, 0., transform.forward().z
        ).normalize();
        let right = Vec3::new(transform.right().x, 0., transform.right().z).normalize();

        let mut direction = Vec3::ZERO;

        if key.pressed(KeyCode::KeyW) {
            direction += forward;
        }

        if key.pressed(KeyCode::KeyS) {
            direction -= forward;
        }

        if key.pressed(KeyCode::KeyD) {
            direction += right;
        }

        if key.pressed(KeyCode::KeyA) {
            direction -= right;
        }

        if key.pressed(KeyCode::KeyQ) {
            transform.rotate(Quat::from_rotation_y(-time.delta_seconds()));
        }

        if key.pressed(KeyCode::KeyE) {
            transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        }

        if key.pressed(KeyCode::Space) {
            transform.translation.y += time.delta_seconds() * speed;
        }

        if key.pressed(KeyCode::ShiftLeft) {
            transform.translation.y -= time.delta_seconds() * speed;
        }

        if direction.length() > 0.0 {
            transform.translation += direction.normalize() * time.delta_seconds() * speed;
        }
    }
}