use bevy::{
    core_pipeline::core_3d::Camera3d,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    scene::SceneInstanceReady,
    window::PrimaryWindow,
};
// use bevy_obj::ObjPlugin;
use bevy_outline::{OutlineMaterial, OutlinePlugin, OutlineRendered};

fn main() {
    println!(
        "Pan and Orbit Camera is enabled:
    Mouse Right Button: Rotate,
    Mouse Middle Button: Pan,
    Mouse Wheel: Zoom."
    );
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OutlinePlugin)
        .add_systems(Update, (pan_orbit_camera, rotate))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ambient_light: ResMut<AmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
) {
    let outline_black = outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::linear_rgb(0.0, 0.0, 0.0).into(),
    });

    let outline_white = outlines.add(OutlineMaterial {
        width: 3.,
        color: Color::linear_rgb(1.0, 1.0, 1.0).into(),
    });

    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::default()))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.8, 0.7, 0.8))),
        Transform::from_xyz(2.0, 0.5, 0.0),
        OutlineRendered,
        MeshMaterial3d(outline_black.clone()),
    ));

    // Sphere
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Sphere::default()))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.3, 0.2, 0.1))),
        Transform::from_xyz(-2.0, 0.5, 0.0),
        OutlineRendered,
        MeshMaterial3d(outline_white.clone()),
    ));

    // Torus
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Torus::default()))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.2, 0.5))),
        Transform::from_xyz(6.0, 0.5, 0.0),
        OutlineRendered,
        MeshMaterial3d(outline_white.clone()),
    ));

    // Monkey head
    commands
        .spawn(SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("head.glb")),
        ))
        .observe(
            |trigger: Trigger<SceneInstanceReady>,
             q_children: Query<&Children>,
             q_mesh: Query<Entity, With<Mesh3d>>,
             mut materials: ResMut<Assets<StandardMaterial>>,
             mut outlines: ResMut<Assets<OutlineMaterial>>,
             mut commands: Commands| {
                if let Some(entity) = q_children
                    .iter_descendants(trigger.entity())
                    .filter(|e| q_mesh.get(*e).is_ok())
                    .next()
                {
                    commands.entity(entity).insert((
                        MeshMaterial3d(materials.add(Color::linear_rgb(0.7, 0.2, 0.5))),
                        Transform::from_xyz(-6.0, 0.5, 0.0),
                        OutlineRendered,
                        MeshMaterial3d(outlines.add(OutlineMaterial {
                            width: 5.,
                            color: Color::linear_rgb(0.7, 0.0, 0.9).into(),
                        })),
                    ));
                }
            },
        );

    // Light
    ambient_light.brightness = 100.0;

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 2_000_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // camera
    let camera_translation = Vec3::new(0.0, 6.0, 12.0);
    let radius = camera_translation.length();
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_translation).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
}

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.read() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        let Projection::Perspective(projection) = projection else {
            continue;
        };
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let Ok(window) = q_window.get_single().map(|w| w.size()) else {
                continue;
            };
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let Ok(window) = q_window.get_single().map(|w| w.size()) else {
                continue;
            };
            pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

fn rotate(time: Res<Time>, mut q_transform: Query<&mut Transform, With<OutlineRendered>>) {
    for mut t in q_transform.iter_mut() {
        t.rotate(Quat::from_rotation_x(time.delta_secs()));
    }
}
