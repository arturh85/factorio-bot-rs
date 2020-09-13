use crate::types::ChunkPosition;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, RgbaImage};
use std::sync::Arc;
// use std::time::Instant;

const TILE_WIDTH: u32 = 256;
const TILE_HEIGHT: u32 = 256;

use crate::factorio::world::FactorioWorld;
use actix_web::{web, HttpResponse};
use std::path::Path;

pub async fn map_tiles(
    world: web::Data<Arc<FactorioWorld>>,
    info: web::Path<(i32, i32, i32)>,
) -> Result<HttpResponse, actix_web::Error> {
    let (tile_z, tile_x, tile_y) = info.into_inner();

    // let started = Instant::now();
    let ((top_left_x, top_left_y), (bottom_right_x, _bottom_right_y)) =
        chunk_zoom(tile_z, tile_x, tile_y);
    let mut buffer: RgbaImage = image::ImageBuffer::new(TILE_WIDTH, TILE_HEIGHT);

    // let chunks = _script_output.chunks.lock().expect("failed to lock mutex");
    // match chunks.get(&chunk_position) {
    //     Some(_chunk) => {
    // info!("chunk found at {:?}: {:?}", &chunk_position, _chunk);
    // paint_single_color(imgbuf, image::Rgb([255, 255, 255]));
    for (_x, _y, pixel) in buffer.enumerate_pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255u8]);
    }

    // let trees = image::Rgba([34u8, 177u8, 76u8, 255u8]);
    // let red = image::Rgba([255u8, 0u8, 0u8, 255u8]);
    // let black = image::Rgba([0u8, 0u8, 0u8, 255u8]);
    let chunks_in_row: f64 = bottom_right_x - top_left_x;
    // let font = Vec::from(include_bytes!("../data/DejaVuSans.ttf") as &[u8]);
    // let font = Vec::from(include_bytes!("../data/FiraMono-Medium.ttf") as &[u8]);
    // let font = Font::try_from_vec(font).unwrap();
    let chunk_width = TILE_WIDTH as f64 / chunks_in_row;
    // if a chunk is not even one pixel, no details shown
    if chunk_width > 1.0 {
        for chunk_ix in 0..(chunks_in_row.ceil() as u32) {
            for chunk_iy in 0..(chunks_in_row.ceil() as u32) {
                let chunk_position = ChunkPosition {
                    x: top_left_x.floor() as i32 + chunk_ix as i32,
                    y: top_left_y.floor() as i32 + chunk_iy as i32,
                };
                let chunk_px = (chunk_ix as f64 * chunk_width).floor() as i32;
                let chunk_py = (chunk_iy as f64 * chunk_width).floor() as i32;

                let graphics_path_str = format!(
                    "workspace/client1/script-output/tiles/tile{}_{}.png",
                    chunk_position.x * 32,
                    chunk_position.y * 32
                );
                let img = match world.image_cache.get_one(&graphics_path_str) {
                    Some(img) => Some(img),
                    None => {
                        let mut writer = world.image_cache_writer.lock().unwrap();
                        // if let Some(img) = writer.get_one(&graphics_path_str) {
                        //     drop(writer);
                        //     world.image_cache.get_one(&graphics_path_str)
                        // } else {
                        let graphics_path = Path::new(&graphics_path_str);
                        if graphics_path.exists() {
                            let img = image::open(graphics_path).unwrap().into_rgba();
                            writer.insert(graphics_path_str.clone(), Box::new(img));
                            writer.refresh();
                            drop(writer);
                            world.image_cache.get_one(&graphics_path_str)
                        } else {
                            None
                        }
                        // }
                    }
                };

                if let Some(img) = img {
                    // let img =
                    //     image::imageops::crop(&mut img, 0, 0, graphic.width, graphic.height);

                    let img = image::imageops::resize(
                        &**img,
                        chunk_width as u32,
                        chunk_width as u32,
                        FilterType::Nearest,
                    );
                    image::imageops::overlay(&mut buffer, &img, chunk_px as u32, chunk_py as u32);
                }
            }
        }
    }
    let dynamic = DynamicImage::ImageRgba8(buffer);
    let mut buf: Vec<u8> = Vec::new();
    dynamic
        .write_to(&mut buf, ImageFormat::Png)
        .expect("failed to write image");
    // info!("image writing took <yellow>{:?}</>", started.elapsed());
    // Content(ContentType::PNG, buf)

    Ok(HttpResponse::Ok().content_type("image/png").body(buf))
}

pub fn chunk_zoom(z: i32, x: i32, y: i32) -> ((f64, f64), (f64, f64)) {
    // one chunk is 32x32 positions big
    let map_size_chunks = 32f64; // map must be a certain size
    let map_size_chunks_half = map_size_chunks / 2.0; // map must be a certain size

    // from -16 to +16

    let x = x as f64;
    let y = y as f64;

    // z = 0, zoom_width = 32
    // z = 1, zoom_width = 16

    // -8 = -16 + (1 * 8)
    // +8 = -16 + (0 * 8)

    let zoom_width = map_size_chunks / 2.0f64.powi(z);
    let top_left = (
        (-map_size_chunks_half + (zoom_width * x)) as f64,
        (-map_size_chunks_half + (zoom_width * y)) as f64,
    );
    let bottom_right = (
        (-map_size_chunks_half + (zoom_width * (x + 1.0f64))) as f64,
        (-map_size_chunks_half + (zoom_width * (y + 1.0f64))) as f64,
    );

    (top_left, bottom_right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_zoom_0() {
        let (zoom_world_top_left, zoom_world_bottom_right) = chunk_zoom(0, 0, 0);
        assert_eq!(zoom_world_top_left, (-16.0, -16.0));
        assert_eq!(zoom_world_bottom_right, (16.0, 16.0));
    }

    #[test]
    fn test_chunk_zoom_1() {
        let (zoom_world_top_left, zoom_world_bottom_right) = chunk_zoom(1, 0, 0);
        assert_eq!(zoom_world_top_left, (-16.0, -16.0));
        assert_eq!(zoom_world_bottom_right, (0.0, 0.0));
    }
}
