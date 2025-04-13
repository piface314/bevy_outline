use std::ops::Deref;

use bevy::{ecs::system::Resource, prelude::*};

use crate::{OutlineMaterial, OutlineRendered};

/// `OutlineMaterial` handle resource used when object is hovered.
/// If this resource does not exist in world, no outline will show.
#[derive(Deref, Resource)]
pub struct HoverOutline(pub Handle<OutlineMaterial>);

/// `OutlineMaterial` handle resource used when object is selected.
/// If this resource does not exist in world, no outline will show.
#[derive(Deref, Resource)]
pub struct SelectedOutline(pub Handle<OutlineMaterial>);

/// `OutlineMaterial` handle resource used when object is pressed or clicked.
/// If this resource does not exist in world, no outline will show.
#[derive(Deref, Resource)]
pub struct PressedOutline(pub Handle<OutlineMaterial>);

/// Outline picking plugin as an alternative to `HighlightablePickingPlugin` in `bevy_mod_picking`
pub struct OutlinePickingPlugin;

impl Plugin for OutlinePickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_observer(update_material_on::<Pointer<Over>, HoverOutline>)
            .add_observer(clear_material_on::<Pointer<Out>>)
            .add_observer(update_material_on::<Pointer<Down>, PressedOutline>)
            .add_observer(update_material_on::<Pointer<Up>, HoverOutline>);
    }
}

fn update_material_on<E, T>(
    trigger: Trigger<E>,
    outline: Option<Res<T>>,
    mut commands: Commands,
    q_outline: Query<Entity, With<OutlineRendered>>,
) where
    E: Event,
    T: Deref<Target = Handle<OutlineMaterial>> + Resource + Send + Sync + 'static,
{
    let Ok(entity) = q_outline.get(trigger.entity()) else {
        return;
    };
    let Some(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };
    if let Some(material) = outline.map(|res| res.clone()) {
        entity_commands.insert(MeshMaterial3d(material));
    }
}

fn clear_material_on<E>(
    trigger: Trigger<E>,
    mut commands: Commands,
    q_outline: Query<Entity, With<OutlineRendered>>,
) where
    E: Event,
{
    let Ok(entity) = q_outline.get(trigger.entity()) else {
        return;
    };
    let Some(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };
    entity_commands.remove::<MeshMaterial3d<OutlineMaterial>>();
}
