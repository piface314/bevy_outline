use bevy::{
    ecs::{
        change_detection::DetectChanges,
        query::{ROQueryItem, With},
        system::{lifetimeless::SRes, Query, SystemParamItem},
        world::Ref,
    },
    math::Vec2,
    prelude::{Commands, Res, ResMut, Resource},
    render::{
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{BindGroup, BindGroupEntry, Buffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        Extract,
    },
    window::{PrimaryWindow, Window},
};
use bytemuck::cast_slice;

use crate::OutlinePipeline;

#[derive(Resource)]
pub(crate) struct ExtractedWindowSize {
    width: f32,
    height: f32,
}

#[derive(ShaderType)]
pub(crate) struct DoubleReciprocalWindowSizeUniform {
    size: Vec2,
}

#[derive(Resource)]
pub(crate) struct DoubleReciprocalWindowSizeMeta {
    pub buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}

pub(crate) fn extract_window_size(
    mut commands: Commands,
    q_window: Extract<Query<Ref<Window>, With<PrimaryWindow>>>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };
    if !window.is_changed() {
        return;
    }
    let width = window.width();
    let height = window.height();
    commands.insert_resource(ExtractedWindowSize { width, height });
}

pub(crate) fn prepare_window_size(
    window_size: Res<ExtractedWindowSize>,
    window_size_meta: ResMut<DoubleReciprocalWindowSizeMeta>,
    render_queue: Res<RenderQueue>,
) {
    if window_size.is_changed() {
        let window_size_uniform = DoubleReciprocalWindowSizeUniform {
            size: Vec2::new(2.0 / window_size.width, 2.0 / window_size.height),
        };
        render_queue.write_buffer(
            &window_size_meta.buffer,
            0,
            cast_slice(&[window_size_uniform.size]),
        )
    }
}

pub(crate) fn queue_window_size_bind_group(
    render_device: Res<RenderDevice>,
    mut double_reciprocal_window_size_meta: ResMut<DoubleReciprocalWindowSizeMeta>,
    pipeline: Res<OutlinePipeline>,
) {
    let bind_group = render_device.create_bind_group(
        Some("window size bind group"),
        &pipeline.window_size_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: double_reciprocal_window_size_meta
                .buffer
                .as_entire_binding(),
        }],
    );
    double_reciprocal_window_size_meta.bind_group = Some(bind_group);
}

pub(crate) struct SetWindowSizeBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetWindowSizeBindGroup<I> {
    type Param = SRes<DoubleReciprocalWindowSizeMeta>;
    type ItemQuery = ();
    type ViewQuery = ();

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let window_size_bind_group = param.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, window_size_bind_group, &[]);
        RenderCommandResult::Success
    }
}
