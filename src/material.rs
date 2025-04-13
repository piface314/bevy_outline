use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialBindGroupId,
    prelude::*,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{AsBindGroup, AsBindGroupError, BindGroup},
        renderer::RenderDevice,
    },
};

use crate::OutlinePipeline;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct OutlineMaterial {
    #[uniform(0)]
    pub width: f32,
    #[uniform(1)]
    pub color: LinearRgba,
}

impl Material for OutlineMaterial {}

pub struct PreparedOutlineMaterial {
    pub bind_group: BindGroup,
}

impl RenderAsset for PreparedOutlineMaterial {
    type SourceAsset = OutlineMaterial;

    type Param = (
        SRes<RenderDevice>,
        SRes<OutlinePipeline>,
        <OutlineMaterial as AsBindGroup>::Param,
    );

    fn prepare_asset(
        material: Self::SourceAsset,
        (render_device, pipeline, ref mut material_param): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        match material.as_bind_group(&pipeline.material_layout, render_device, material_param) {
            Ok(prepared) => Ok(PreparedOutlineMaterial {
                bind_group: prepared.bind_group,
            }),
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(material))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}

impl PreparedOutlineMaterial {
    pub fn get_bind_group_id(&self) -> MaterialBindGroupId {
        MaterialBindGroupId(Some(self.bind_group.id()))
    }
}
