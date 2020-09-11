use crate::types::{Position, Rect};
use image::{DynamicImage, ImageFormat, RgbaImage};
use imageproc::drawing::draw_hollow_rect_mut;
use std::sync::Arc;
// use std::time::Instant;

const TILE_WIDTH: u32 = 256;
const TILE_HEIGHT: u32 = 256;

use crate::draw::draw_arrow_mut;
use crate::factorio::world::FactorioWorld;
use actix_web::{web, HttpResponse};
use petgraph::visit::EdgeRef;

pub async fn entity_graph_tiles(
    world: web::Data<Arc<FactorioWorld>>,
    info: web::Path<(i32, i32, i32)>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut buffer = create_tile();
    let bounding_box = tile_boundaries(info.0, info.1, info.2);
    let base_x = bounding_box.left_top.x();
    let base_y = bounding_box.left_top.y();
    let scaling_factor = TILE_WIDTH as f64 / bounding_box.width();
    for (entity, rect, id) in world.entity_graph.inner_tree().query(bounding_box.into()) {
        let width = (rect.size.width as f64 * scaling_factor).round() as u32;
        let height = (rect.size.height as f64 * scaling_factor).round() as u32;
        if width > 0 && height > 0 {
            let draw_rect = imageproc::rect::Rect::at(
                ((rect.origin.x as f64 - base_x) * scaling_factor).round() as i32,
                ((rect.origin.y as f64 - base_y) * scaling_factor).round() as i32,
            )
            .of_size(width, height);
            match world.entity_graph.node_by_id(&id) {
                Some(node_id) => {
                    draw_hollow_rect_mut(
                        &mut buffer,
                        draw_rect,
                        image::Rgba([3u8, 169u8, 244u8, 255u8]),
                    );
                    let graph = world.entity_graph.inner_graph();
                    for edge in graph.edges_directed(node_id, petgraph::Direction::Outgoing) {
                        if let Some(node) = graph.node_weight(edge.target()) {
                            draw_arrow_mut(
                                &mut buffer,
                                (
                                    ((entity.position.x() - base_x) * scaling_factor) as f32,
                                    ((entity.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                (
                                    ((node.position.x() - base_x) * scaling_factor) as f32,
                                    ((node.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                image::Rgba([76u8, 175u8, 80u8, 255u8]),
                                scaling_factor / 10.,
                            );
                        }
                    }
                    for edge in graph.edges_directed(node_id, petgraph::Direction::Incoming) {
                        if let Some(node) = graph.node_weight(edge.source()) {
                            draw_arrow_mut(
                                &mut buffer,
                                (
                                    ((node.position.x() - base_x) * scaling_factor) as f32,
                                    ((node.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                (
                                    ((entity.position.x() - base_x) * scaling_factor) as f32,
                                    ((entity.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                image::Rgba([76u8, 175u8, 80u8, 255u8]),
                                scaling_factor / 10.,
                            );
                        }
                    }
                }
                None => {
                    draw_hollow_rect_mut(
                        &mut buffer,
                        draw_rect,
                        image::Rgba([255u8, 0u8, 0u8, 255u8]),
                    );
                }
            }
        }
    }

    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(build_image_body(buffer)))
}

pub async fn flow_graph_tiles(
    world: web::Data<Arc<FactorioWorld>>,
    info: web::Path<(i32, i32, i32)>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut buffer = create_tile();
    let bounding_box = tile_boundaries(info.0, info.1, info.2);
    let base_x = bounding_box.left_top.x();
    let base_y = bounding_box.left_top.y();
    let scaling_factor = TILE_WIDTH as f64 / bounding_box.width();
    // info!("bounding_box: {:?}", bounding_box);
    for (entity, rect, _id) in world.entity_graph.inner_tree().query(bounding_box.into()) {
        let width = (rect.size.width as f64 * scaling_factor).round() as u32;
        let height = (rect.size.height as f64 * scaling_factor).round() as u32;
        if width > 0 && height > 0 {
            let draw_rect = imageproc::rect::Rect::at(
                ((rect.origin.x as f64 - base_x) * scaling_factor).round() as i32,
                ((rect.origin.y as f64 - base_y) * scaling_factor).round() as i32,
            )
            .of_size(width, height);
            match world.flow_graph.node_at(&entity.position) {
                Some(node_id) => {
                    draw_hollow_rect_mut(
                        &mut buffer,
                        draw_rect,
                        image::Rgba([3u8, 169u8, 244u8, 255u8]),
                    );
                    let graph = world.flow_graph.inner_graph();
                    for edge in graph.edges_directed(node_id, petgraph::Direction::Outgoing) {
                        if let Some(node) = graph.node_weight(edge.target()) {
                            draw_arrow_mut(
                                &mut buffer,
                                (
                                    ((entity.position.x() - base_x) * scaling_factor) as f32,
                                    ((entity.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                (
                                    ((node.position.x() - base_x) * scaling_factor) as f32,
                                    ((node.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                image::Rgba([76u8, 175u8, 80u8, 255u8]),
                                scaling_factor / 10.,
                            );
                        }
                    }
                    for edge in graph.edges_directed(node_id, petgraph::Direction::Incoming) {
                        if let Some(node) = graph.node_weight(edge.source()) {
                            draw_arrow_mut(
                                &mut buffer,
                                (
                                    ((node.position.x() - base_x) * scaling_factor) as f32,
                                    ((node.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                (
                                    ((entity.position.x() - base_x) * scaling_factor) as f32,
                                    ((entity.position.y() - base_y) * scaling_factor) as f32,
                                ),
                                image::Rgba([76u8, 175u8, 80u8, 255u8]),
                                scaling_factor / 10.,
                            );
                        }
                    }
                }
                None => {
                    draw_hollow_rect_mut(
                        &mut buffer,
                        draw_rect,
                        image::Rgba([255u8, 0u8, 0u8, 255u8]),
                    );
                }
            }
        }
    }
    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(build_image_body(buffer)))
}

pub fn tile_boundaries(z: i32, x: i32, y: i32) -> Rect {
    // one chunk is 32x32 positions big
    let map_size_chunks = 32f64; // map must be a certain size
    let map_size_chunks_half = map_size_chunks / 2.0; // map must be a certain size
    let x = x as f64;
    let y = y as f64;
    let zoom_width = map_size_chunks / 2.0f64.powi(z);
    let top_left = (
        (-map_size_chunks_half + (zoom_width * x)) as f64,
        (-map_size_chunks_half + (zoom_width * y)) as f64,
    );
    let bottom_right = (
        (-map_size_chunks_half + (zoom_width * (x + 1.0f64))) as f64,
        (-map_size_chunks_half + (zoom_width * (y + 1.0f64))) as f64,
    );
    Rect::new(
        &Position::new(top_left.0 * 32., top_left.1 * 32.),
        &Position::new(bottom_right.0 * 32., bottom_right.1 * 32.),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_boundaries_0() {
        let Rect {
            left_top,
            right_bottom,
        } = tile_boundaries(0, 0, 0);
        assert_eq!(left_top, Position::new(-512.0, -512.0));
        assert_eq!(right_bottom, Position::new(512.0, 512.0));
    }

    #[test]
    fn test_tile_boundaries_1() {
        let Rect {
            left_top,
            right_bottom,
        } = tile_boundaries(1, 0, 0);
        assert_eq!(left_top, Position::new(-512.0, -512.0));
        assert_eq!(right_bottom, Position::new(0.0, 0.0));
    }
}

pub fn create_tile() -> RgbaImage {
    let mut buffer: RgbaImage = image::ImageBuffer::new(TILE_WIDTH, TILE_HEIGHT);
    for (_x, _y, pixel) in buffer.enumerate_pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255u8]);
    }
    buffer
}

pub fn build_image_body(buffer: RgbaImage) -> Vec<u8> {
    let dynamic = DynamicImage::ImageRgba8(buffer);
    let mut buf: Vec<u8> = Vec::new();
    dynamic
        .write_to(&mut buf, ImageFormat::Png)
        .expect("failed to write image");
    buf
}
