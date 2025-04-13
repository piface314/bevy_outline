use bevy::{
    core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey},
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, RenderMaterialInstances, RenderMeshInstances,
        SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        mesh::{MeshVertexBufferLayoutRef, RenderMesh},
        render_asset::RenderAssets,
        render_phase::{
            BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases,
        },
        render_resource::{
            AsBindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState,
            BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
            DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PipelineCache,
            PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderStages, ShaderType,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines,
            StencilFaceState, StencilState, TextureFormat, VertexState,
        },
        renderer::RenderDevice,
        view::{ExtractedView, RenderVisibleEntities},
    },
};

use crate::{
    DoubleReciprocalWindowSizeUniform, OutlineMaterial, OutlineRendered, PreparedOutlineMaterial,
    SetWindowSizeBindGroup, ATTRIBUTE_OUTLINE_NORMAL,
};

#[derive(Resource)]
pub struct OutlinePipeline {
    pub(crate) mesh_pipeline: MeshPipeline,
    pub(crate) material_layout: BindGroupLayout,
    pub(crate) window_size_layout: BindGroupLayout,
    pub(crate) shader_handle: Handle<Shader>,
}

impl FromWorld for OutlinePipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = MeshPipeline::from_world(world);
        let render_device = world.resource::<RenderDevice>();
        let material_layout = OutlineMaterial::bind_group_layout(render_device);
        let window_size_layout = render_device.create_bind_group_layout(
            Some("window size layout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(DoubleReciprocalWindowSizeUniform::min_size()),
                },
                count: None,
            }],
        );
        let shader_handle = world.resource::<AssetServer>().add(Shader::from_wgsl(
            include_str!("render/outline.wgsl"),
            "render/outline_3d.wgsl",
        ));
        Self {
            mesh_pipeline,
            material_layout,
            window_size_layout,
            shader_handle,
        }
    }
}

impl SpecializedMeshPipeline for OutlinePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let vertex_attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_OUTLINE_NORMAL.at_shader_location(1),
        ];
        let shader_defs = vec![
            "MESH_PIPELINE".into(),
            "VERTEX_OUTPUT_INSTANCE_INDEX".into(),
        ];
        let vertex_buffer_layout = layout.0.get_layout(&vertex_attributes)?;

        let view_layout = self.mesh_pipeline.get_view_layout(key.into()).clone();

        let mesh_layout = self.mesh_pipeline.mesh_layouts.model_only.clone();

        let bind_group_layout = vec![
            view_layout,
            mesh_layout,
            self.material_layout.clone(),
            self.window_size_layout.clone(),
        ];

        Ok(RenderPipelineDescriptor {
            label: Some("outline_mesh_pipeline".into()),
            layout: bind_group_layout,
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader_handle.clone(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![vertex_buffer_layout],
            },
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Front),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: self.shader_handle.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: true,
        })
    }
}

pub(crate) type OutlinePipelineCommands = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform at bind group 0
    SetMeshViewBindGroup<0>,
    // Set the mesh uniform at bind group 1
    SetMeshBindGroup<1>,
    // Set the material uniform at bind group 2
    SetOutlineMaterialBindGroup<2>,
    // Set the window size uniform at bind group 3
    SetWindowSizeBindGroup<3>,
    // Draw the mesh
    DrawMesh,
);

pub(crate) struct SetOutlineMaterialBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetOutlineMaterialBindGroup<I> {
    type Param = (
        SRes<RenderAssets<PreparedOutlineMaterial>>,
        SRes<RenderMaterialInstances<OutlineMaterial>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (materials, material_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let materials = materials.into_inner();
        let material_instances = material_instances.into_inner();

        let Some(material_asset_id) = material_instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(material) = materials.get(*material_asset_id) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &material.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub(crate) fn queue_outlines(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_material_instances: Res<RenderMaterialInstances<OutlineMaterial>>,
    render_materials: Res<RenderAssets<PreparedOutlineMaterial>>,
    outline_pipeline: Res<OutlinePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<OutlinePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    views: Query<(Entity, &RenderVisibleEntities, &Msaa), With<ExtractedView>>,
) {
    let draw_function_id = opaque_3d_draw_functions
        .read()
        .id::<OutlinePipelineCommands>();

    for (view_entity, view_visible_entities, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

        for &(render_entity, visible_entity) in
            view_visible_entities.get::<With<OutlineRendered>>().iter()
        {
            let Some(material_asset_id) = render_material_instances.get(&visible_entity) else {
                continue;
            };

            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(visible_entity)
            else {
                continue;
            };

            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let Some(material) = render_materials.get(*material_asset_id) else {
                continue;
            };

            let mesh_key = view_key
                | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());

            let pipeline_id = match pipelines.specialize(
                &pipeline_cache,
                &outline_pipeline,
                mesh_key,
                &mesh.layout,
            ) {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    return;
                }
            };

            mesh_instance
                .material_bind_group_id
                .set(material.get_bind_group_id());

            opaque_phase.add(
                Opaque3dBinKey {
                    draw_function: draw_function_id,
                    pipeline: pipeline_id,
                    asset_id: mesh_instance.mesh_asset_id.into(),
                    material_bind_group_id: material.get_bind_group_id().0,
                    lightmap_image: None,
                },
                (render_entity, visible_entity),
                BinnedRenderPhaseType::BatchableMesh,
            );
        }
    }
}
