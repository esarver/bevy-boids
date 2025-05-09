use bevy::prelude::*;
use boids::BoidsPlugin;

pub mod boids {
    use avian3d::prelude::*;
    use bevy::{
        math::{DQuat, DVec4},
        prelude::*,
    };
    pub struct BoidsPlugin;

    impl Plugin for BoidsPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(PhysicsPlugins::default())
                .insert_resource(AmbientLight {
                    brightness: 2.0,
                    ..default()
                })
                .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
                .insert_resource(Gravity(Vec3::ZERO))
                .add_systems(Startup, spawn_boids::<3>);
        }
    }

    #[derive(Component)]
    struct Boid;

    fn spawn_boids<const N: u32>(
        mut cmds: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for n in 0..N {
            cmds.spawn((
                Boid,
                RigidBody::Dynamic,
                Collider::cone(0.25, 1.0),
                Mesh3d(meshes.add(Cone::new(0.125, 0.5))),
                MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
                Transform::from_xyz(0.0 + 1.0 * n as f32, 0.0 + 1.0 * n as f32, 0.0),
                LinearVelocity(Vec3::new(0.0, 1.0, 0.0)),
            ));
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsPlugin)
        .add_systems(Startup, spawn_lights)
        .add_systems(Update, || {})
        .run();
}

fn spawn_lights(mut cmds: Commands) {
    cmds.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    cmds.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));
}
