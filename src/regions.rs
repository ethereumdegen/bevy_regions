use crate::regionmap::SubRegionMapU8;
use bevy::asset::{AssetPath, LoadState};
use bevy::pbr::{ExtendedMaterial, OpaqueRendererMethod};
use bevy::prelude::*;
use bevy::render::render_resource::{
    TextureFormat,
};

use bevy::utils::HashMap;

//use crate::chunk::{Chunk, ChunkCoordinates, ChunkCoords, ChunkData, TerrainMaterialExtension};

use crate::regions_config::RegionsConfig;
use crate::regions_material::{RegionsMaterial, RegionsMaterialExtension, ToolPreviewUniforms};
//use crate::terrain_material::{ChunkMaterialUniforms, TerrainMaterial};

//use crate::terrain_config::TerrainConfig;

/*


Chunks should be more persistent

each chunk should have its own heightmap and splat map !!!  these are their own files too.



*/



#[derive(Resource, Default)]
pub struct RegionsDataMapResource {
    pub regions_data_map: Option<SubRegionMapU8>, // Keyed by chunk id
}



#[derive(Component)]
pub struct RegionPlaneMesh {

}



//attach me to camera
//#[derive(Component, Default)]
//pub struct TerrainViewer {}

#[derive(Default, PartialEq, Eq)]
pub enum RegionsImageDataLoadStatus {
    //us this for texture image and splat image and alpha mask .. ?
    #[default]
    NotLoaded,
    Loaded,
    NeedsReload,
}

#[derive(Default, PartialEq, Eq)]
pub enum RegionsDataStatus {
    //us this for texture image and splat image and alpha mask .. ?
    #[default]
    NotLoaded,
    Loaded,
}

#[derive(Component, Default)]
pub struct  RegionsData {
    // pub chunk_entity_lookup: HashMap<u32,Entity>,  //why is this necessary  ??
    // pub terrain_config: TerrainConfig,
    pub regions_data_status: RegionsDataStatus,

    texture_image_handle: Option<Handle<Image>>,
    color_map_texture_handle:  Option<Handle<Image>>,
 
    regions_image_data_load_status: bool 
     // meshes: Res <Assets<Mesh>>
}

impl RegionsData {
    pub fn new() -> Self {
        let regions_data = RegionsData::default();

         
        regions_data
    }
}



pub type PlanarPbrBundle = MaterialMeshBundle<RegionsMaterialExtension>;


pub fn initialize_regions(
    mut commands: Commands,

    mut asset_server: ResMut<AssetServer>, 

    mut regions_query: Query<(Entity, &mut RegionsData, &RegionsConfig)>,

    mut meshes: ResMut <Assets<Mesh>>,
    mut region_materials: ResMut<Assets<RegionsMaterialExtension>>,

    mut images: ResMut<Assets<Image>>
) {
    for (region_entity, mut regions_data, regions_config) in regions_query.iter_mut() {
        if regions_data.regions_data_status ==  RegionsDataStatus::NotLoaded {
                

         


             if regions_data.color_map_texture_handle.is_none() {
                 regions_data.color_map_texture_handle = Some( 
                    asset_server.load( 
                        regions_config.region_color_map_texture_path.clone() 
                     ) );

           }


             if regions_data.regions_image_data_load_status == false {continue};



             let regions_texture = regions_data.get_regions_texture_image().clone();


             let regions_material: Handle<RegionsMaterialExtension> =
                region_materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        // can be used in forward or deferred mode.
                        opaque_render_method: OpaqueRendererMethod::Auto,
                       // alpha_mode: AlphaMode::Opaque,

                        reflectance: 0.0,
                        perceptual_roughness: 0.9,
                        specular_transmission: 0.1,

                        //base_color_texture: regions_data.color_map_texture_handle.clone(),

                        // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
                        // in forward mode, the output can also be modified after lighting is applied.
                        // see the fragment shader `extended_material.wgsl` for more info.
                        // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                        // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                        ..Default::default()
                    },
                    extension: RegionsMaterial {
                         
                        tool_preview_uniforms: ToolPreviewUniforms::default(),
                        regions_texture: regions_texture.clone(),
                        color_map_texture: regions_data.color_map_texture_handle.clone(),
                      
                        ..default()
                    },
                });

           let dimensions = regions_config.boundary_dimensions.clone();

             // ground plane
           let regions_plane = commands.spawn(PlanarPbrBundle {
                mesh: meshes.add(Plane3d::default().mesh().size( dimensions.x, dimensions.y )),
                material: regions_material,
                ..default()
            }).id();

            commands.entity(  region_entity  ).add_child(  regions_plane ) ;



            //do regionmap load_from_file .. ? 

            regions_data.regions_data_status = RegionsDataStatus::Loaded
        }
    }
}

impl RegionsData {
    pub fn get_regions_texture_image(&self) -> &Option<Handle<Image>> {
        &self.texture_image_handle
    }

      
 
}


pub fn load_regions_texture_from_image(
    mut regions_query: Query<(&mut RegionsData, &RegionsConfig)>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut regions_data, regions_config) in regions_query.iter_mut() {
        if regions_data.texture_image_handle.is_none() {
            let texture_path = &regions_config.region_texture_path;
            let tex_image = asset_server.load(AssetPath::from_path(texture_path));
            regions_data.texture_image_handle = Some(tex_image);
        }

        if regions_data.regions_image_data_load_status ==false {
            let texture_image: &mut Image = match &regions_data.texture_image_handle {
                Some(texture_image_handle) => {
                    let texture_image_loaded = asset_server.get_load_state(texture_image_handle);

                    if texture_image_loaded != Some(LoadState::Loaded) {
                        println!("regions texture not yet loaded");
                        continue;
                    }

                    images.get_mut(texture_image_handle).unwrap()
                }
                None => continue,
            };

            // Specify the desired texture format
            let desired_format = TextureFormat::Rgba8Uint;


            texture_image.texture_descriptor.format = desired_format; 
            // Create a new texture descriptor with the desired format
           // let mut texture_descriptor = TextureDescriptor

             

            regions_data.regions_image_data_load_status =true;
        }
    }
}


//consider building a custom loader for this , not  Image
/*pub fn load_regions_texture_from_image(
    mut terrain_query: Query<(&mut TerrainData, &TerrainConfig)>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    
) {
    for (mut terrain_data, terrain_config) in terrain_query.iter_mut() {
        if terrain_data.texture_image_handle.is_none() {
            let array_texture_path = &terrain_config.diffuse_folder_path;

            let tex_image = asset_server.load(AssetPath::from_path(array_texture_path));
            terrain_data.texture_image_handle = Some(tex_image);
        }

        //try to load the height map data from the height_map_image_handle
        if !terrain_data.texture_image_finalized {
            let texture_image: &mut Image = match &terrain_data.texture_image_handle {
                Some(texture_image_handle) => {
                    let texture_image_loaded = asset_server.get_load_state(texture_image_handle);

                    if texture_image_loaded != Some(LoadState::Loaded) {
                        println!("terrain texture not yet loaded");
                        continue;
                    }

                    images.get_mut(texture_image_handle).unwrap()
                }
                None => continue,
            };

            //https://github.com/bevyengine/bevy/pull/10254
            texture_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                label: None,
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                address_mode_w: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                ..default()
            });

            // Create a new array texture asset from the loaded texture.
            let desired_array_layers = terrain_config.texture_image_sections;

            let need_to_reinterpret = desired_array_layers > 1
                && texture_image.texture_descriptor.size.depth_or_array_layers == 1;

            if need_to_reinterpret {
                //info!("texture info {:?}" , texture_image.texture_descriptor.dimension, texture_image.size().depth_or_array_layers);

                texture_image.reinterpret_stacked_2d_as_array(desired_array_layers);
            }

           

            terrain_data.texture_image_finalized = true;
        }
    }
}

pub fn load_terrain_normal_from_image(
    mut terrain_query: Query<(&mut TerrainData, &TerrainConfig)>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    //  materials: Res <Assets<TerrainMaterialExtension>>,
) {
    for (mut terrain_data, terrain_config) in terrain_query.iter_mut() {
        if terrain_data.normal_image_handle.is_none() {
            let normal_texture_path = &terrain_config.normal_folder_path;

            let tex_image = asset_server.load(AssetPath::from_path(normal_texture_path));
            terrain_data.normal_image_handle = Some(tex_image);
        }

        
        if !terrain_data.normal_image_finalized {
            let texture_image: &mut Image = match &terrain_data.normal_image_handle {
                Some(texture_image_handle) => {
                    let texture_image_loaded = asset_server.get_load_state(texture_image_handle);

                    if texture_image_loaded != Some(LoadState::Loaded) {
                        println!("terrain texture not yet loaded");
                        continue;
                    }

                    images.get_mut(texture_image_handle).unwrap()
                }
                None => continue,
            };

            //https://github.com/bevyengine/bevy/pull/10254
            texture_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                label: None,
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                address_mode_w: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                ..default()
            });

            // Create a new array texture asset from the loaded texture.
            let desired_array_layers = terrain_config.texture_image_sections;

            let need_to_reinterpret = desired_array_layers > 1
                && texture_image.texture_descriptor.size.depth_or_array_layers == 1;

            if need_to_reinterpret {
                //info!("texture info {:?}" , texture_image.texture_descriptor.dimension, texture_image.size().depth_or_array_layers);

                texture_image.reinterpret_stacked_2d_as_array(desired_array_layers);
            }

            

            terrain_data.normal_image_finalized = true;
        }
    }
}
*/