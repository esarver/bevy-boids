use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::prelude::*;
//use bevy_inspector_egui::bevy_egui::EguiPlugin;
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
            .insert_resource(ClearColor(Color::srgb_u8(0, 0, 0)))
            .insert_resource(BoidRng(SmallRng::from_rng(&mut rand::rng())))
            .add_systems(Startup, (spawn_tank, spawn_boids::<1000>))
            .add_systems(Update, show_tank_bounds)
            .add_systems(Update, ((alignment, cohesion, separation), step).chain());
        }
    }

    #[derive(Resource)]
    pub struct BoidRng(SmallRng);

    #[derive(Component)]
    pub struct Tank;

    #[derive(Component)]
    pub struct Boid;

    #[derive(Component)]
    pub struct Perception {
        radius: f32,
    }

    #[derive(Component)]
    pub struct AlignmentDir(Dir3);

    #[derive(Component)]
    pub struct CohesionDir(Dir3);

    #[derive(Component)]
    pub struct SeparationDir(Dir3);

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
    ) {
        cmds.spawn((
            Tank,
            Mesh3d(meshes.add(Cuboid::new(TANK_WIDTH, TANK_HEIGHT, TANK_DEPTH))),
            //MeshMaterial3d(materials.add(Color::srgba_u8(0, 0, 255, 255/3))),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }

    pub fn show_tank_bounds(mut gizmos: Gizmos) {
        gizmos.cuboid(
            Transform::IDENTITY.with_scale(Vec3::new(TANK_WIDTH, TANK_HEIGHT, TANK_DEPTH)),
            Color::srgb(0.25, 0.25, 0.25),
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
            let fx: f32 = rng.0.random_range(-1.0..1.0);
            let fy: f32 = rng.0.random_range(-1.0..1.0);
            let fz: f32 = rng.0.random_range(-1.0..1.0);
            let la_up: Dir3 = Vec3::new(fx, fy, fz)
                .try_into()
                .expect("should convert from rng to Dir3");
            let la_fw: Dir3 = la_up
                .any_orthogonal_vector()
                .try_into()
                .expect("should convert from rng to Dir3");
            cmds.spawn((
                Boid,
                Mesh3d(meshes.add(Cone::new(BOID_LENGTH / 4.0, BOID_LENGTH))),
                MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.5))),
                Transform::from_xyz(x, y, z).looking_to(la_fw, la_up),
                Speed {
                    max: 10.0,
                    current: 10.0,
                },
                Perception { radius: 20.0 },
                AlignmentDir(la_up),
                CohesionDir(la_up),
                SeparationDir(la_up),
            ));
        }
    }

    pub fn alignment(
        mut query: Query<(Entity, &Transform, &Perception, &mut AlignmentDir), With<Boid>>,
        others: Query<(Entity, &Transform), With<Boid>>,
    ) {
        for (id, t, p, mut la) in &mut query {
            let mut num_others = 0;
            let mut accum_rot = Vec3::ZERO;
            for (o_id, o_t) in &others {
                if o_id == id {
                    continue;
                }
                if t.translation.distance(o_t.translation) > p.radius {
                    continue;
                }
                accum_rot += *o_t.up();
                num_others += 1;
            }
            if num_others > 0 {
                la.0 = match (accum_rot / num_others as f32).normalize().try_into() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Alignment: Encountered '{e}', falling back to original value");

                        t.up()
                    }
                };
                // TODO
                //let avg_speed = accum_speed / num_others as f32;
                //s.current = if avg_speed > s.max { s.max } else { avg_speed };
            }
        }
    }

    pub fn cohesion(
        mut query: Query<(Entity, &Transform, &Perception, &mut CohesionDir), With<Boid>>,
        others: Query<(Entity, &Transform), With<Boid>>,
    ) {
        for (id, t, p, mut cd) in &mut query {
            let mut num_others = 0;
            let mut accum_other_pos = Vec3::ZERO;
            for (o_id, o_t) in &others {
                if o_id == id {
                    continue;
                }
                if t.translation.distance(o_t.translation) > p.radius {
                    continue;
                }

                accum_other_pos += o_t.translation;
                num_others += 1;
            }
            if num_others > 0 {
                cd.0 = match (accum_other_pos / num_others as f32).normalize().try_into() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Cohesion: Encountered '{e}', falling back to original value");
                        cd.0
                    }
                };
            }
        }
    }

    pub fn separation(
        mut query: Query<(Entity, &Transform, &mut SeparationDir), With<Boid>>,
        others: Query<(Entity, &Transform), With<Boid>>,
    ) {
        for (id, t, mut cd) in &mut query {
            let mut num_others = 0;
            let mut accum_other_pos = Vec3::ZERO;
            for (o_id, o_t) in &others {
                if o_id == id {
                    continue;
                }
                if t.translation.distance(o_t.translation) < 10.0 {
                    continue;
                }

                accum_other_pos -= o_t.translation;
                num_others += 1;
            }
            if num_others > 0 {
                cd.0 = match (accum_other_pos / num_others as f32).normalize().try_into() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Cohesion: Encountered '{e}', falling back to original value");
                        cd.0
                    }
                };
            }
        }
    }

    pub fn step(
        time: Res<Time>,
        mut query: Query<
            (
                &mut Transform,
                &Speed,
                &AlignmentDir,
                &CohesionDir,
                &SeparationDir,
            ),
            With<Boid>,
        >,
    ) {
        for (mut t, s, ad, cd, sd) in &mut query {
            // Using "up" because cone points are in that direction.
            let t_forward = t.forward();
            let t_up = t.up();
            let look_dir = ad.0.as_vec3() + cd.0.as_vec3() + sd.0.as_vec3();
            t.look_to(
                *t_forward.slerp(
                    match ad.0.any_orthonormal_vector().try_into() {
                        Ok(v) => v,
                        Err(_) => t_forward,
                    },
                    time.delta_secs(),
                ),
                *t_up.slerp(
                    look_dir.try_into().expect("Should convert from lookdir"),
                    time.delta_secs(),
                ),
            );
            t.translation = t.translation + (t.up() * s.current * time.delta_secs());

            // Magically wrap space for the tank a la pacman
            if t.translation.x > TANK_WIDTH / 2.0 {
                t.translation.x = -(TANK_WIDTH / 2.0);
            }
            if t.translation.x < -(TANK_WIDTH / 2.0) {
                t.translation.x = TANK_WIDTH / 2.0;
            }
            if t.translation.y > TANK_HEIGHT / 2.0 {
                t.translation.y = -(TANK_HEIGHT / 2.0);
            }
            if t.translation.y < -(TANK_HEIGHT / 2.0) {
                t.translation.y = TANK_HEIGHT / 2.0;
            }
            if t.translation.z > TANK_DEPTH / 2.0 {
                t.translation.z = -(TANK_DEPTH / 2.0);
            }
            if t.translation.z < -(TANK_DEPTH / 2.0) {
                t.translation.z = TANK_DEPTH / 2.0;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsPlugin)
        //.add_plugins(EguiPlugin {
        //    enable_multipass_for_primary_context: true,
        //})
        //.add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 12.0,
                    ..default()
                },
                text_color: Color::srgb(0.0, 1.0, 0.0),
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
            },
        })
        .register_type::<Speed>()
        .add_systems(Startup, spawn_lights)
        .add_systems(Update, || {})
        .run();
}

fn spawn_lights(mut cmds: Commands) {
    cmds.spawn((
        DirectionalLight {
            color: Color::srgb_u8(255, 255, 255),
            ..default()
        },
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
