use crate::factorio::output_parser::FactorioWorld;
use crate::types::{ChunkPosition, FactorioGraphic, Position};
use evmap::ReadGuard;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, ImageFormat, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_text_mut};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;
use rusttype::{Font, Scale};
use std::sync::Arc;
// use std::time::Instant;

use actix_web::{web, HttpResponse};
use std::path::Path;

pub async fn map_tiles(
    world: web::Data<Arc<FactorioWorld>>,
    info: web::Path<(i32, i32, i32)>,
) -> Result<HttpResponse, actix_web::Error> {
    let tile_z = info.0;
    let tile_x = info.1;
    let tile_y = info.2;

    // let started = Instant::now();
    let ((top_left_x, top_left_y), (bottom_right_x, _bottom_right_y)) =
        chunk_zoom(tile_z, tile_x, tile_y);
    let img_width = 256;
    let img_height = 256;
    let mut buffer: RgbaImage = image::ImageBuffer::new(img_width, img_height);

    // let chunks = _script_output.chunks.lock().expect("failed to lock mutex");
    // match chunks.get(&chunk_position) {
    //     Some(_chunk) => {
    // info!("chunk found at {:?}: {:?}", &chunk_position, _chunk);
    // paint_single_color(imgbuf, image::Rgb([255, 255, 255]));
    for (_x, _y, pixel) in buffer.enumerate_pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255u8]);
    }

    let trees = image::Rgba([34u8, 177u8, 76u8, 255u8]);
    let red = image::Rgba([255u8, 0u8, 0u8, 255u8]);
    let black = image::Rgba([0u8, 0u8, 0u8, 255u8]);
    let chunks_in_row: f64 = bottom_right_x - top_left_x;
    // let font = Vec::from(include_bytes!("../data/DejaVuSans.ttf") as &[u8]);
    let font = Vec::from(include_bytes!("../data/FiraMono-Medium.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();
    let chunk_width = img_width as f64 / chunks_in_row;
    // if a chunk is not even one pixel, no details shown
    if chunk_width > 1.0 {
        let factor = chunk_width / 32.0;
        // if a position is not even one pixel, no details shown
        if factor > 1.0 {
            for chunk_ix in 0..(chunks_in_row.ceil() as u32) {
                for chunk_iy in 0..(chunks_in_row.ceil() as u32) {
                    let chunk_position = ChunkPosition {
                        x: top_left_x.floor() as i32 + chunk_ix as i32,
                        y: top_left_y.floor() as i32 + chunk_iy as i32,
                    };
                    let chunk_ix: f64 = chunk_position.x as f64 - top_left_x;
                    let chunk_iy: f64 = chunk_position.y as f64 - top_left_y;
                    let chunk_px = (chunk_ix * chunk_width).floor() as i32;
                    let chunk_py = (chunk_iy * chunk_width).floor() as i32;
                    match world.chunks.get_one(&chunk_position) {
                        Some(chunk) => {
                            if !chunk.objects.is_empty() {
                                // info!("chunk found at {:?}: {:?}", &chunk_position, _chunk);
                            }
                            for (tile_idx, tile) in chunk.tiles.iter().enumerate() {
                                let x_mod: f64 = (tile_idx % 32) as f64;
                                let y_mod: f64 = (tile_idx / 32) as f64;
                                let rect_x = (chunk_px + (x_mod * factor).round() as i32) as i32;
                                let rect_y = (chunk_py + (y_mod * factor).round() as i32) as i32;
                                let name = match tile.name.find('-') {
                                    Some(pos) => {
                                        if &tile.name[0..pos] == "red" {
                                            match tile.name[pos + 1..].find('-') {
                                                Some(pos2) => &tile.name[pos + 1..pos + pos2 + 1],
                                                None => &tile.name[pos + 1..],
                                            }
                                        } else {
                                            &tile.name[0..pos]
                                        }
                                    }
                                    None => &tile.name,
                                };
                                draw_filled_rect_mut(
                                    &mut buffer,
                                    Rect::at(rect_x, rect_y).of_size(factor as u32, factor as u32),
                                    match &name[..] {
                                        "sand" => image::Rgba([255u8, 249u8, 15u8, 255u8]),
                                        "desert" => image::Rgba([255u8, 229u8, 15u8, 255u8]),
                                        "dry" => image::Rgba([255u8, 255u8, 128u8, 255u8]),
                                        "dirt" => image::Rgba([172u8, 255u8, 0u8, 255u8]),
                                        "grass" => image::Rgba([0u8, 255u8, 64u8, 255u8]),
                                        "water" => image::Rgba([0u8, 162u8, 232u8, 255u8]),
                                        "deepwater" => image::Rgba([18u8, 16u8, 254u8, 255u8]),
                                        _ => {
                                            warn!(
                                                "<red>unhandled tile type</>: <yellow>{}</> to <bright-blue>'{}'</>",
                                                &tile.name,
                                                name
                                            );
                                            image::Rgba([255u8, 0u8, 255u8, 255u8])
                                        }
                                    },
                                );
                                if tile.player_collidable
                                    && (&name[..] != "water" && &name[..] != "deepwater")
                                {
                                    draw_hollow_rect_mut(
                                        &mut buffer,
                                        Rect::at(rect_x, rect_y)
                                            .of_size(factor as u32, factor as u32),
                                        image::Rgba([255u8, 0u8, 0u8, 255u8]),
                                    );
                                }
                            }
                            for object in &chunk.objects {
                                let (x_mod, y_mod) =
                                    chunk_offset(&chunk_position, &object.bounding_box.left_top);
                                let mut rect_x =
                                    (chunk_px + (x_mod * factor).round() as i32) as i32;
                                let mut rect_y =
                                    (chunk_py + (y_mod * factor).round() as i32) as i32;

                                let graphic: Option<ReadGuard<FactorioGraphic>> =
                                    world.graphics.get_one(&object.name);
                                // let graphic = world.graphics.get_one(&object.name);
                                // info!(
                                //     "object {} @ {}/{} -> {:?}",
                                //     object.name,
                                //     object.position.x(),
                                //     object.position.y(),
                                //     graphic
                                // );
                                match graphic {
                                    Some(graphic) => {
                                        // let mut image_cache = image_cache.lock().unwrap();
                                        let mut img =
                                            world.image_cache.get_one(&graphic.image_path);
                                        if img.is_none() {
                                            // __base__/graphics/decorative/rock-huge/hr-rock-huge-05.png
                                            let prefix_pos = graphic.image_path.find('/').unwrap();
                                            let graphics_path = format!(
                                                "workspace/server/data/base/{}",
                                                &graphic.image_path[prefix_pos + 1..]
                                            );
                                            let graphics_path = Path::new(&graphics_path);
                                            if graphics_path.exists() {
                                                // draw_hollow_rect_mut(
                                                //     &mut buffer,
                                                //     Rect::at(rect_x, rect_y)
                                                //         .of_size(factor as u32, factor as u32),
                                                //     red,
                                                // );
                                                let loaded_img =
                                                    image::open(graphics_path).unwrap().into_rgba();
                                                let mut writer =
                                                    world.image_cache_writer.lock().unwrap();
                                                writer.insert(
                                                    graphic.image_path.clone(),
                                                    Box::new(loaded_img),
                                                );
                                                writer.refresh();
                                                drop(writer);
                                                img =
                                                    world.image_cache.get_one(&graphic.image_path);
                                            }
                                        }
                                        let mut img = *img.unwrap().clone();
                                        let img = image::imageops::crop(
                                            &mut img,
                                            0,
                                            0,
                                            graphic.width,
                                            graphic.height,
                                        );
                                        let mut img = image::imageops::resize(
                                            &img,
                                            (object.bounding_box.width() * factor).ceil() as u32,
                                            (object.bounding_box.height() * factor).ceil() as u32,
                                            FilterType::Nearest,
                                        );
                                        // info!(
                                        //     "overlay {} at {}, {}",
                                        //     object.name,
                                        //     rect_x,
                                        //     rect_y
                                        // );
                                        let mut w = img.width();
                                        let mut h = img.height();
                                        let mut img = if rect_x < 0 && w > (-rect_x as u32) {
                                            w -= -rect_x as u32;
                                            let sub_image = image::imageops::crop(
                                                &mut img,
                                                (-rect_x) as u32,
                                                0,
                                                w,
                                                h,
                                            );
                                            rect_x = 0;
                                            sub_image
                                        } else {
                                            img.sub_image(0, 0, w, h)
                                        };
                                        let img = if rect_y < 0 && h > (-rect_y as u32) {
                                            h -= -rect_y as u32;
                                            let sub_image = image::imageops::crop(
                                                &mut img,
                                                0,
                                                -rect_y as u32,
                                                w,
                                                h,
                                            );
                                            rect_y = 0;
                                            sub_image
                                        } else {
                                            image::imageops::crop(&mut img, 0, 0, w, h)
                                        };
                                        image::imageops::overlay(
                                            &mut buffer,
                                            &img,
                                            rect_x as u32,
                                            rect_y as u32,
                                        );
                                        draw_hollow_rect_mut(
                                            &mut buffer,
                                            Rect::at(rect_x, rect_y).of_size(w, h),
                                            red,
                                        );
                                    }
                                    None => {
                                        // log::info!("no graphic for {:?}", &object);
                                        let name = match object.name.find('-') {
                                            Some(pos) => &object.name[0..pos],
                                            None => &object.name,
                                        };
                                        match &name[..] {
                                            "tree" => {
                                                if factor > 1.0 {
                                                    draw_filled_circle_mut(
                                                        &mut buffer,
                                                        (rect_x, rect_y),
                                                        factor as i32,
                                                        trees,
                                                    );
                                                }
                                            }
                                            _ => {
                                                let width = (object.bounding_box.width() * factor)
                                                    .ceil()
                                                    as u32;
                                                let height = (object.bounding_box.height() * factor)
                                                    .ceil()
                                                    as u32;
                                                if width > 0 && height > 0 {
                                                    draw_filled_rect_mut(
                                                        &mut buffer,
                                                        Rect::at(rect_x, rect_y)
                                                            .of_size(width, height),
                                                        red,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                };
                            }
                            for resource in &chunk.resources {
                                let (x_mod, y_mod) =
                                    chunk_offset(&chunk_position, &resource.position);
                                let rect_x = (chunk_px + (x_mod * factor).round() as i32) as i32;
                                let rect_y = (chunk_py + (y_mod * factor).round() as i32) as i32;
                                draw_hollow_rect_mut(
                                    &mut buffer,
                                    Rect::at(rect_x, rect_y).of_size(factor as u32, factor as u32),
                                    match &resource.name[..] {
                                        "uranium-ore" => image::Rgba([169u8, 241u8, 18u8, 255u8]),
                                        "iron-ore" => image::Rgba([79u8, 119u8, 174u8, 255u8]),
                                        "copper-ore" => image::Rgba([232u8, 105u8, 21u8, 255u8]),
                                        "coal" => image::Rgba([22u8, 22u8, 22u8, 255u8]),
                                        "stone" => image::Rgba([162u8, 122u8, 32u8, 255u8]),
                                        "crude-oil" => image::Rgba([0u8, 0u8, 0u8, 255u8]),
                                        _ => {
                                            warn!(
                                                "<red>unhandled resource type</>: <bright-blue>{}</>",
                                                resource.name
                                            );
                                            image::Rgba([255u8, 0u8, 255u8, 255u8])
                                        }
                                    },
                                );
                            }

                            draw_hollow_rect_mut(
                                &mut buffer,
                                Rect::at(chunk_px, chunk_py)
                                    .of_size(chunk_width as u32, chunk_width as u32),
                                black,
                            );
                            let height = 4.0 * factor as f32;
                            let scale = Scale {
                                x: height * 2.0,
                                y: height,
                            };
                            draw_text_mut(
                                &mut buffer,
                                black,
                                chunk_px as u32,
                                chunk_py as u32,
                                scale,
                                &font,
                                &format!("{}/{}", chunk_position.x, chunk_position.y),
                            );
                        }
                        None => {
                            draw_filled_rect_mut(
                                &mut buffer,
                                Rect::at(
                                    (chunk_ix as u32 * chunk_width.round() as u32) as i32,
                                    (chunk_iy as u32 * chunk_width.round() as u32) as i32,
                                )
                                .of_size(chunk_width as u32, chunk_width as u32),
                                black,
                            );
                            draw_hollow_rect_mut(
                                &mut buffer,
                                Rect::at(
                                    (chunk_ix as u32 * chunk_width.round() as u32) as i32,
                                    (chunk_iy as u32 * chunk_width.round() as u32) as i32,
                                )
                                .of_size(chunk_width as u32, chunk_width as u32),
                                black,
                            );
                        }
                    }
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

fn chunk_offset(chunk_position: &ChunkPosition, position: &Position) -> (f64, f64) {
    let mut x_mod: f64 = position.x() - (chunk_position.x * 32) as f64;
    let mut y_mod: f64 = position.y() - (chunk_position.y * 32) as f64;
    if x_mod < 0.0 {
        x_mod = 32.0 - x_mod.abs()
    }
    if y_mod < 0.0 {
        y_mod = 32.0 - y_mod.abs()
    }
    (x_mod, y_mod)
}

// fn paint_single_color(&mut imgbuf: RgbImage, color: Rgb) {
//     for (_x, _y, pixel) in imgbuf.enumerate_pixels_mut() {
//         *pixel = color
//     }
// }
