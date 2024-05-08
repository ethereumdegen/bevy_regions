use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use thiserror::Error;

/*
https://github.com/norman784/gaiku/blob/master/crates/gaiku_baker_heightmap/src/lib.rs
*/

#[derive(Error, Debug)]
pub enum RegionMapError {
    #[error("failed to load the image")]
    LoadingError,
}

pub type RegionMapU16 = Vec<Vec<u16>>;

pub struct SubRegionMapU16(pub Vec<Vec<u16>>);

impl SubRegionMapU16 {
    pub fn from_regionmap_u16(
        regionmap: &RegionMapU16,
        
        bounds_pct: [[f32; 2]; 2],
    ) -> Self {
        let width = regionmap.len() - 0;
        let height = regionmap[0].len() - 0;

        // let start_bound = [ (width as f32 * bounds_pct[0][0]) as usize, (height as f32 * bounds_pct[0][1]) as usize  ];
        //let end_bound = [ (width as f32 * bounds_pct[1][0]) as usize , (height as f32 * bounds_pct[1][1]) as usize   ];

        let start_bound = [
            (width as f32 * bounds_pct[0][0]).ceil() as usize,
            (height as f32 * bounds_pct[0][1]).ceil() as usize,
        ];

        //really need to load 1 extra row than we normally would think we would... so here it is
        let end_bound = [
            (width as f32 * bounds_pct[1][0]).ceil() as usize + 1,
            (height as f32 * bounds_pct[1][1]).ceil() as usize + 1,
        ];

        let mut pixel_data = Vec::new();

        for x in start_bound[0]..end_bound[0] {
            if x >= width {
                continue;
            }

            let mut row = Vec::new();
            for y in start_bound[1]..end_bound[1] {
                if y >= height {
                    continue;
                }

                row.push(regionmap[x][y]);
            }
            pixel_data.push(row);
        }

        SubRegionMapU16(pixel_data)
    }

    pub fn append_x_row(&mut self, row: Vec<u16>) {
        self.0.push(row);
    }

    //this is busted ? \
    pub fn append_y_col(&mut self, col: Vec<u16>) {
        // Check if the number of elements in `col` matches the number of rows in the height data.
        // If not, you may need to handle this discrepancy based on your specific requirements.
        if col.len() != self.0.len() {
            // Handle error or discrepancy.
            // For example, you might return early or panic, depending on how strict you want to be.
            // e.g., panic!("Column length does not match the number of rows in height data.");
            println!("WARN: cannot append y col "); // Or handle this situation appropriately.
            panic!("Column length does not match the number of rows in height data.");
        }

        for (row, &value) in self.0.iter_mut().zip(col.iter()) {
            row.push(value);
        }
    }
}

pub trait RegionMap {
    fn load_from_image(image: &Image) -> Result<Box<Self>, RegionMapError>;
}

impl RegionMap for RegionMapU16 {
    fn load_from_image(image: &Image) -> Result<Box<Self>, RegionMapError> {
        let width = image.size().x as usize;
        let height = image.size().y as usize;

        let format = image.texture_descriptor.format;

        if format != TextureFormat::R16Uint {
            println!("regionmap: wrong format {:?}", format);
            return Err(RegionMapError::LoadingError);
        }

        //maybe somehow fail if the format is not R16uint

        // With the format being R16Uint, each pixel is represented by 2 bytes
        let mut region_map = Vec::with_capacity(height);

        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let index = 2 * (y * width + x); // 2 because of R16Uint
                let height_value = u16::from_le_bytes([image.data[index], image.data[index + 1]]);
                row.push(height_value);
            }
            region_map.push(row);
        }

        Ok(Box::new(region_map))
    }
 
}
