use std::ops::{Add, Div, Neg};
use std::path::PathBuf;

use bevy::ecs::entity::Entity;
use bevy::math::Vec2;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::{AssetServer, Assets};
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::texture::Image;

use bevy::prelude::*;
 
use core::fmt::{self, Display, Formatter};

 
use crate::regionmap::SubRegionMapU16;
use crate::regions::{RegionsData, RegionsDataMapResource};
use crate::regions_config::RegionsConfig;
use crate::regions_material::RegionsMaterialExtension;

 
 
use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use serde_json;

use rand::Rng;

use core::cmp::{max, min};


pub struct BevyRegionEditsPlugin {
    
}

impl Default for BevyRegionEditsPlugin {
    fn default() -> Self {
        Self {
             
        }
    }
}
impl Plugin for BevyRegionEditsPlugin {
    fn build(&self, app: &mut App) {




      app.add_event::<EditRegionEvent>();
       app.add_event::<RegionCommandEvent>();
       app.add_event::<RegionBrushEvent>();
        app.add_systems(Update, apply_tool_edits); //put this in a sub plugin ?
        app.add_systems(Update, apply_command_events);


    }
}

#[derive(Debug, Clone)]
pub enum EditingTool {
    SetRegionMap { region_index: u16 },        // height, radius, save to disk
   // SetSplatMap { r: u8, g: u8, b: u8 }, //R, G, B, radius, save to disk
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum BrushType {
    #[default]
    SetExact, // hardness ?
    Smooth,
    //Noise,
    EyeDropper,
}

impl Display for BrushType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            BrushType::SetExact => "SetExact",
            BrushType::Smooth => "Smooth",
          //  BrushType::Noise => "Noise",
            BrushType::EyeDropper => "EyeDropper",
        };

        write!(f, "{}", label)
    }
}

// entity, editToolType, coords, magnitude
#[derive(Event, Debug, Clone)]
pub struct EditRegionEvent {
   // pub entity: Entity,
    pub tool: EditingTool,
    pub radius: f32,
    pub brush_hardness: f32, //1.0 is full
    pub coordinates: Vec2,
    pub brush_type: BrushType,
}

#[derive(Event, Debug, Clone)]
pub enum RegionBrushEvent {
    EyeDropRegionIndex { region_index: u16 },
  //  EyeDropSplatMap { r: u8, g: u8, b: u8 },
}

#[derive(Event, Debug, Clone)]
pub enum RegionCommandEvent {
    SaveAll ,  
}

pub fn apply_command_events(
    asset_server: Res<AssetServer>,

   // mut chunk_query: Query<(&Chunk, &mut ChunkData, &Parent, &Children)>, //chunks parent should have terrain data

    mut images: ResMut<Assets<Image>>,
    mut region_materials: ResMut<Assets<RegionsMaterialExtension>>,

    mut region_maps_res: ResMut<RegionsDataMapResource>, //like height map resource 

    region_data_query: Query<(&RegionsData, &RegionsConfig)>,

    //chunk_mesh_query: Query<(Entity, &Handle<Mesh>, &GlobalTransform), With<TerrainChunkMesh>>,
   // meshes: Res<Assets<Mesh>>,

    mut ev_reader: EventReader<RegionCommandEvent>,
) {
    for ev in ev_reader.read() {
       
            //let region_entity_id = parent_terrain_entity.get();

            //if region_data_query.get(region_entity_id).is_ok() == false {
            //    continue;
            //}

            let (terrain_data, terrain_config) = region_data_query.get_single().ok() else {continue};



            match ev {
                RegionCommandEvent::SaveAll => {
                    let file_name = format!("{}.png", chunk.chunk_id);
                    let asset_folder_path = PathBuf::from("assets");
                    if *save_height {
                        if let Some(chunk_height_data) =
                            chunk_height_maps.chunk_height_maps.get(&chunk.chunk_id)
                        {
                            save_chunk_height_map_to_disk(
                                chunk_height_data,
                                asset_folder_path
                                    .join(&terrain_config.height_folder_path)
                                    .join(&file_name),
                            );
                        }
                    }
 

                     

                    println!("save complete");
                }
            }
        }
     

    //  Ok(())
}

pub fn apply_tool_edits(
    mut asset_server: Res<AssetServer>,

   region_data_query: Query<(&RegionsData, &RegionsConfig)> , 

    mut images: ResMut<Assets<Image>>,
    mut region_materials: ResMut<Assets<RegionsMaterialExtension>>,
  

    mut region_map_data_res: ResMut<RegionsDataMapResource>,

  //  terrain_query: Query<(&TerrainData, &TerrainConfig)>,

    mut ev_reader: EventReader<EditRegionEvent>,

    mut evt_writer: EventWriter<RegionBrushEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- region edit event!", &ev.tool);

        let intersected_entity = &ev.entity;

        //  if let Some((chunk, mut chunk_data)) = chunk_query.get_mut(intersected_entity.clone()).ok()
        if let Some((chunk_entity, _)) = chunk_mesh_query.get(intersected_entity.clone()).ok() {
            let mut chunk_entities_within_range: Vec<Entity> = Vec::new();

            let mut chunk_dimensions = [256, 256]; //compute me from terrain config
            if let Some((_, _, _, terrain_entity, _)) =
                chunk_query.get_mut(chunk_entity.get().clone()).ok()
            {
                if let Some((terrain_data, terrain_config)) =
                    terrain_query.get(terrain_entity.get().clone()).ok()
                {
                    let chunk_rows = terrain_config.chunk_rows;
                    let terrain_dimensions = terrain_config.terrain_dimensions;

                    chunk_dimensions = [
                        terrain_dimensions.x as u32 / chunk_rows,
                        terrain_dimensions.y as u32 / chunk_rows,
                    ];
                }
            }

            //populate chunk_entities_within_range
            for (chunk_entity, _, _, _, chunk_transform) in chunk_query.iter() {
                let tool_coords: &Vec2 = &ev.coordinates;
                let chunk_transform = chunk_transform.translation();
                let chunk_transform_vec2: Vec2 = Vec2::new(chunk_transform.x, chunk_transform.z);

                let chunk_dimensions_vec: Vec2 =
                    Vec2::new(chunk_dimensions.x() as f32, chunk_dimensions.y() as f32);
                let chunk_center_transform =
                    chunk_transform_vec2.add(chunk_dimensions_vec.div(2.0));

                let chunk_local_distance = tool_coords.distance(chunk_center_transform);

                if chunk_local_distance < 800.0 {
                    chunk_entities_within_range.push(chunk_entity);
                }
            }

            //compute average height since we need this for some tools

            let mut total_height: f32 = 0.0;
            let mut heights_len = 0;

            for chunk_entity_within_range in chunk_entities_within_range.clone() {
                if let Some((
                    chunk_entity,
                    chunk,
                    mut chunk_data,
                    terrain_entity,
                    chunk_transform,
                )) = chunk_query.get_mut(chunk_entity_within_range.clone()).ok()
                {
                    if let Some(height_map_data) =
                        &mut chunk_height_maps.chunk_height_maps.get_mut(&chunk.chunk_id)
                    {
                        let tool_coords: &Vec2 = &ev.coordinates;
                        let chunk_transform = chunk_transform.translation();
                        let chunk_transform_vec2: Vec2 =
                            Vec2::new(chunk_transform.x, chunk_transform.z);

                        let tool_coords_local = tool_coords.add(chunk_transform_vec2.neg());

                        //need to make an array of all of the data indices of the terrain that will be set .. hm ?
                        let img_data_length = height_map_data.0.len();

                        //let mut height_changed = false;
                        let radius = &ev.radius;
                        //   let radius_clone = radius.clone();

                        //  let tool_height:f32 = *height as f32;
                        for x in 0..img_data_length {
                            for y in 0..img_data_length {
                                let local_coords = Vec2::new(x as f32, y as f32);
                                if tool_coords_local.distance(local_coords) < *radius {
                                    let original_height = height_map_data.0[x][y];
                                    total_height += original_height as f32;
                                    heights_len += 1;
                                }
                            }
                        }
                    }
                }
            }
            let average_height = total_height as f32 / heights_len as f32;
            // ------
            let radius = &ev.radius;
            let brush_type = &ev.brush_type;

            let brush_hardness = &ev.brush_hardness;
            //apply the tool to each chunk in range
            for chunk_entity_within_range in chunk_entities_within_range {
                if let Some((
                    chunk_entity,
                    chunk,
                    mut chunk_data,
                    terrain_entity,
                    chunk_transform,
                )) = chunk_query.get_mut(chunk_entity_within_range.clone()).ok()
                {
                    //   if let Some(mut terrain_data) = terrain_data_query.get_mut(terrain_entity.get().clone()).ok() { //why cant i find this ?

                    match &ev.tool {
                        EditingTool::SetRegionMap { region_index } => {
                            if let Some(height_map_data) =
                                &mut chunk_height_maps.chunk_height_maps.get_mut(&chunk.chunk_id)
                            {

                                
                                // if let Some(img) = images.get_mut( height_map_image_handle ){

                                let tool_coords: &Vec2 = &ev.coordinates;
                                let chunk_transform = chunk_transform.translation();
                                let chunk_transform_vec2: Vec2 =
                                    Vec2::new(chunk_transform.x, chunk_transform.z);

                                let tool_coords_local = tool_coords.add(chunk_transform_vec2.neg());

                                //need to make an array of all of the data indices of the terrain that will be set .. hm ?
                                let img_data_length = height_map_data.0.len();

                                let mut height_changed = false;

                                let radius_clone = radius.clone();

                                match brush_type {
                                    BrushType::SetExact => {
                                        for x in 0..img_data_length {
                                            for y in 0..img_data_length {
                                                let local_coords = Vec2::new(x as f32, y as f32);

                                                let hardness_multiplier = get_hardness_multiplier(
                                                    tool_coords_local.distance(local_coords),
                                                    radius_clone,
                                                    *brush_hardness,
                                                );
                                                let original_height = height_map_data.0[x][y];

                                                if tool_coords_local.distance(local_coords)
                                                    < radius_clone
                                                {
                                                    let new_height = height.clone();
                                                    height_map_data.0[x][y] =
                                                        apply_hardness_multiplier(
                                                            original_height as f32,
                                                            new_height as f32,
                                                            hardness_multiplier,
                                                        )
                                                            as u16;
                                                    height_changed = true;
                                                }
                                            }
                                        }
                                    }

                                    BrushType::Smooth => {
                                        for x in 0..img_data_length {
                                            for y in 0..img_data_length {
                                                let local_coords = Vec2::new(x as f32, y as f32);
                                                if tool_coords_local.distance(local_coords)
                                                    < *radius
                                                {
                                                    let hardness_multiplier =
                                                        get_hardness_multiplier(
                                                            tool_coords_local
                                                                .distance(local_coords),
                                                            radius_clone,
                                                            *brush_hardness,
                                                        );

                                                    let original_height = height_map_data.0[x][y];
                                                    // Gather heights of the current point and its neighbors within the brush radius

                                                    let new_height = ((average_height
                                                        + original_height as f32)
                                                        / 2.0)
                                                        as u16;
                                                    height_map_data.0[x][y] =
                                                        apply_hardness_multiplier(
                                                            original_height as f32,
                                                            new_height as f32,
                                                            hardness_multiplier,
                                                        )
                                                            as u16;
                                                    height_changed = true;
                                                }
                                            }
                                        }
                                    }

                                    BrushType::Noise => {
                                        let mut rng = rand::thread_rng();
                                        for x in 0..img_data_length {
                                            for y in 0..img_data_length {
                                                let local_coords = Vec2::new(x as f32, y as f32);
                                                if tool_coords_local.distance(local_coords)
                                                    < *radius
                                                {
                                                    let original_height = height_map_data.0[x][y];
                                                    let hardness_multiplier =
                                                        get_hardness_multiplier(
                                                            tool_coords_local
                                                                .distance(local_coords),
                                                            radius_clone,
                                                            *brush_hardness,
                                                        );

                                                    // Generate a random value between -0.5 and 0.5, then scale it by the desired height variation
                                                    let noise = rng.gen::<f32>() - 0.5;
                                                    let noise_scaled = noise * *height as f32; // Adjust *height to control the scale of the noise
                                                    let new_height = noise_scaled as u16;

                                                    height_map_data.0[x][y] =
                                                        apply_hardness_multiplier(
                                                            original_height as f32,
                                                            new_height as f32,
                                                            hardness_multiplier,
                                                        )
                                                            as u16;
                                                    height_changed = true;
                                                }
                                            }
                                        }
                                    }

                                    BrushType::EyeDropper => {
                                        // Check if the clicked coordinates are within the current chunk
                                        if tool_coords.x >= chunk_transform_vec2.x
                                            && tool_coords.x
                                                < chunk_transform_vec2.x
                                                    + chunk_dimensions.x() as f32
                                            && tool_coords.y >= chunk_transform_vec2.y
                                            && tool_coords.y
                                                < chunk_transform_vec2.y
                                                    + chunk_dimensions.y() as f32
                                        {
                                            let tool_coords_local =
                                                tool_coords.add(chunk_transform_vec2.neg());
                                            let x = tool_coords_local.x as usize;
                                            let y = tool_coords_local.y as usize;

                                            if x < img_data_length && y < img_data_length {
                                                let local_height = height_map_data.0[x][y];
                                                evt_writer.send(
                                                    RegionBrushEvent::EyeDropTerrainHeight {
                                                        height: local_height,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }

                                if height_changed {
                                    chunk_data.height_map_image_data_load_status =
                                        RegionImageDataLoadStatus::NeedsReload;
                                }
                            }
                        }

                     



                    } //match
                }
            }
        }
    }
}

fn get_hardness_multiplier(pixel_distance: f32, brush_radius: f32, brush_hardness: f32) -> f32 {
    // Calculate the distance as a percentage of the radius
    let distance_percent = pixel_distance / brush_radius;
    let adjusted_distance_percent = f32::min(1.0, distance_percent); // Ensure it does not exceed 1

    // Calculate the fade effect based on brush hardness
    // When hardness is 0, this will linearly interpolate from 1 at the center to 0 at the edge
    // When hardness is between 0 and 1, it adjusts the fade effect accordingly
    let fade_effect = 1.0 - adjusted_distance_percent;

    // Apply the brush hardness to scale the fade effect, ensuring a minimum of 0
    f32::max(
        0.0,
        fade_effect * (1.0 + brush_hardness) - (adjusted_distance_percent * brush_hardness),
    )
}

fn apply_hardness_multiplier(
    original_height: f32,
    new_height: f32,
    hardness_multiplier: f32,
) -> f32 {
    original_height + (new_height - original_height) * hardness_multiplier
}
