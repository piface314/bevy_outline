#![doc = include_str!("../README.md")]

mod material;
mod pipeline;
mod prepare;
mod smooth_normal;
mod window_size;

#[cfg(feature = "picking")]
pub mod picking;

use bevy::{
    core_pipeline::core_3d::Opaque3d,
    pbr::{extract_mesh_materials, RenderMaterialInstances},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::MeshVertexAttribute,
        render_asset::{prepare_assets, RenderAssetPlugin},
        render_phase::AddRenderCommand,
        render_resource::{BufferDescriptor, BufferUsages, SpecializedMeshPipelines, VertexFormat},
        renderer::RenderDevice,
        view::{self, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
};

pub use material::OutlineMaterial;
use material::PreparedOutlineMaterial;
pub use pipeline::OutlinePipeline;
use pipeline::{queue_outlines, OutlinePipelineCommands};
use prepare::prepare_outline_mesh;
use window_size::{
    extract_window_size, prepare_window_size, queue_window_size_bind_group,
    DoubleReciprocalWindowSizeMeta, DoubleReciprocalWindowSizeUniform, SetWindowSizeBindGroup,
};

pub const ATTRIBUTE_OUTLINE_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("OutlineNormal", 9885409170, VertexFormat::Float32x3);

pub struct OutlinePlugin;

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<OutlineMaterial>()
            .register_type::<MeshMaterial3d<OutlineMaterial>>()
            .add_plugins(RenderAssetPlugin::<PreparedOutlineMaterial>::default())
            .add_plugins(ExtractComponentPlugin::<OutlineRendered>::default())
            .add_systems(PostUpdate, prepare_outline_mesh)
            .add_systems(
                PostUpdate,
                view::check_visibility::<With<OutlineRendered>>
                    .in_set(VisibilitySystems::CheckVisibility),
            );

        // We make sure to add these to the render app, not the main app.
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<RenderMaterialInstances<OutlineMaterial>>()
            .init_resource::<SpecializedMeshPipelines<OutlinePipeline>>()
            .add_render_command::<Opaque3d, OutlinePipelineCommands>()
            .add_systems(
                ExtractSchedule,
                (
                    extract_mesh_materials::<OutlineMaterial>, // NOTE: out of render set?
                    extract_window_size,
                )
                    .in_set(RenderSet::ExtractCommands),
            )
            .add_systems(
                Render,
                prepare_window_size.in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                queue_outlines
                    .in_set(RenderSet::Queue)
                    .after(prepare_assets::<PreparedOutlineMaterial>),
            )
            .add_systems(
                Render,
                queue_window_size_bind_group.in_set(RenderSet::Queue),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_device = app.world().resource::<RenderDevice>();
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("window size uniform buffer"),
            size: size_of::<DoubleReciprocalWindowSizeUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<OutlinePipeline>()
            .insert_resource(DoubleReciprocalWindowSizeMeta {
                buffer,
                bind_group: None,
            });
    }
}

/// A marker component that represents an entity that is to be rendered using
/// the [`OutlinePipeline`].
///
/// Note the [`ExtractComponent`] trait implementation. This is necessary to
/// tell Bevy that this object should be pulled into the render world.
#[derive(Clone, Component, ExtractComponent)]
pub struct OutlineRendered;
