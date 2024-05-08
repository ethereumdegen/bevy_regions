use bevy::pbr::ExtendedMaterial;
use bevy::asset::VisitAssetDependencies;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

use bevy::render::render_asset::RenderAssets;

use bevy::pbr::StandardMaterialFlags;
use bevy::pbr::StandardMaterialUniform;

use bevy::pbr::MaterialExtension;

pub const REGION_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(5433284082028047579);

 pub type RegionsMaterialExtension = ExtendedMaterial<StandardMaterial, RegionsMaterial>;

#[derive(Clone, ShaderType, Default, Debug)]
pub struct ToolPreviewUniforms {
    pub tool_coordinates: Vec2,
    pub tool_radius: f32,
    pub tool_color: Vec3,
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
pub struct RegionsMaterial {
   // #[uniform(20)]
  //  pub chunk_uniforms: ChunkMaterialUniforms,

    #[uniform(21)]
    pub tool_preview_uniforms: ToolPreviewUniforms,

    
    #[texture(22)]  //rgba8unorm 
   // #[sampler(23)]
    pub regions_texture: Option<Handle<Image>>,
 
}

impl MaterialExtension for RegionsMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(REGION_SHADER_HANDLE)
    }

    fn deferred_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(REGION_SHADER_HANDLE)
    }
}
