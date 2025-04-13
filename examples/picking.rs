use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_outline::{
    picking::{HoverOutline, OutlinePickingPlugin, PressedOutline, SelectedOutline},
    OutlineMaterial, OutlinePlugin, OutlineRendered,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(OutlinePlugin)
        .add_plugins(OutlinePickingPlugin)
        .add_systems(Startup, (set_picking_outlines, setup))
        .run();
}

fn set_picking_outlines(mut commands: Commands, mut outlines: ResMut<Assets<OutlineMaterial>>) {
    commands.insert_resource(HoverOutline(outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::linear_rgb(1.0, 1.0, 1.0).into(),
    })));
    commands.insert_resource(SelectedOutline(outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::linear_rgb(1.0, 1.0, 0.2).into(),
    })));
    commands.insert_resource(PressedOutline(outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::linear_rgb(1.0, 1.0, 0.5).into(),
    })));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ambient_light: ResMut<AmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::default()))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.8, 0.7, 0.8))),
        Transform::from_xyz(2.0, 0.5, 0.0),
        OutlineRendered,
    ));

    // Sphere
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Sphere::default()))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.3, 0.2, 0.1))),
        Transform::from_xyz(-2.0, 0.5, 0.0),
        OutlineRendered,
    ));

    // Torus
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Torus::default()))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.2, 0.5))),
        Transform::from_xyz(6.0, 0.5, 0.0),
        OutlineRendered,
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
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_translation).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
