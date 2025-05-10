use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use boid::{BoidsPlugin, Speed};

pub mod boid {
    use bevy::prelude::*;
    use bevy_inspector_egui::prelude::*;
    use rand::{self, Rng, SeedableRng, rngs::SmallRng};

    pub const BOID_LENGTH: f32 = 0.5f32;
    pub const TANK_WIDTH: f32 = 80.0f32;
    pub const TANK_HEIGHT: f32 = 45.0f32;
    pub const TANK_DEPTH: f32 = 45.0f32;

    pub struct BoidsPlugin;

    impl Plugin for BoidsPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(AmbientLight {
                brightness: 2.0,
                ..default()
            })
            .insert_resource(ClearColor(Color::srgb(0.8, 0.8, 0.8)))
            .insert_resource(BoidRng(SmallRng::from_rng(&mut rand::rng())))
            .add_systems(Startup, (spawn_tank, spawn_boids::<50>))
            .add_systems(Update, (step, show_tank_bounds));
        }
    }

    #[derive(Resource)]
    pub struct BoidRng(SmallRng);

    #[derive(Component)]
    pub struct Tank;

    #[derive(Component)]
    pub struct Boid;

    #[derive(Component, Reflect, Default, InspectorOptions)]
    #[reflect(Component, InspectorOptions)]
    pub struct Speed {
        /// The maximum speed of this boid in meters per second
        max: f32,

        /// The current speed of this boid (note, this is the magnitude of the velocity).
        /// The actual velocity should be this value times the rotation.
        #[inspector(min = 0.0, max = 5.0)]
        current: f32,
    }

    pub fn spawn_tank(
        mut cmds: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        cmds.spawn((
            Tank,
            Mesh3d(meshes.add(Cuboid::new(TANK_WIDTH, TANK_HEIGHT, TANK_DEPTH))),
            MeshMaterial3d(materials.add(Color::srgba_u8(0, 0, 255, 128))),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }

    pub fn show_tank_bounds(mut gizmos: Gizmos) {
        gizmos.cuboid(
            Transform::IDENTITY.with_scale(Vec3::new(TANK_WIDTH, TANK_HEIGHT, TANK_DEPTH)),
            Color::BLACK,
        );
    }

    pub fn spawn_boids<const NUM: u32>(
        mut cmds: Commands,
        mut rng: ResMut<BoidRng>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for _ in 0..NUM {
            let x: f32 = rng.0.random_range(-TANK_WIDTH / 2.0..TANK_WIDTH / 2.0);
            let y: f32 = rng.0.random_range(-TANK_HEIGHT / 2.0..TANK_HEIGHT / 2.0);
            let z: f32 = rng.0.random_range(-TANK_DEPTH / 2.0..TANK_DEPTH / 2.0);
            cmds.spawn((
                Boid,
                Mesh3d(meshes.add(Cone::new(BOID_LENGTH / 4.0, BOID_LENGTH))),
                MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
                Transform::from_xyz(x, y, z).looking_to(Dir3::Y, Dir3::X),
                Speed {
                    max: 5.0,
                    current: 5.0,
                },
            ));
        }
    }

    pub fn alignment() {}

    pub fn cohesion() {}

    pub fn separation() {}
    pub fn step(time: Res<Time>, mut query: Query<(&mut Transform, &Speed), With<Boid>>) {
        for (mut t, s) in &mut query {
            // Using "up" because cone points are in that direction.
            t.translation = t.translation + (t.up() * s.current * time.delta_secs());

            info!("{}", t.translation);

            // Magically wrap space for the tank a la pacman
            if t.translation.x > TANK_WIDTH / 2.0 {
                t.translation.x = -(TANK_WIDTH / 2.0);
            }
            if t.translation.x < -(TANK_WIDTH / 2.0) {
                t.translation.x = TANK_WIDTH / 2.0;
            }
            if t.translation.y > TANK_WIDTH / 2.0 {
                t.translation.y = -(TANK_WIDTH / 2.0);
            }
            if t.translation.y < -(TANK_WIDTH / 2.0) {
                t.translation.y = TANK_WIDTH / 2.0;
            }
            if t.translation.z > TANK_WIDTH / 2.0 {
                t.translation.z = -(TANK_WIDTH / 2.0);
            }
            if t.translation.z < -(TANK_WIDTH / 2.0) {
                t.translation.z = TANK_WIDTH / 2.0;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsPlugin)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .register_type::<Speed>()
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
        Transform::from_xyz(0.0, 0.0, boid::TANK_WIDTH).looking_at(Vec3::ZERO, Dir3::Y),
    ));
}
