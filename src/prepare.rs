use bevy::{
    ecs::{change_detection::DetectChanges, query::Changed, world::Ref},
    prelude::{Assets, Mesh, Query, ResMut},
    render::mesh::Mesh3d,
};

use crate::{smooth_normal::smooth_normal, OutlineRendered, ATTRIBUTE_OUTLINE_NORMAL};

pub fn prepare_outline_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    q_mesh: Query<Ref<Mesh3d>, Changed<OutlineRendered>>,
) {
    for mesh_handle in q_mesh.iter().filter(|mesh| mesh.is_changed()) {
        let Some(mesh) = meshes.get_mut(mesh_handle.id()) else {
            continue;
        };
        // Don't have outline normal, just compute it.
        if !mesh.contains_attribute(ATTRIBUTE_OUTLINE_NORMAL) {
            mesh.insert_attribute(ATTRIBUTE_OUTLINE_NORMAL, smooth_normal(mesh));
        }
    }
}
