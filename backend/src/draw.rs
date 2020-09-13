use crate::factorio::util::{vector_add, vector_multiply, vector_normalize, vector_substract};
use crate::types::Position;
use imageproc::drawing::{draw_line_segment_mut, Canvas};

#[allow(clippy::clone_on_copy)]
pub fn draw_arrow_mut<C>(
    canvas: &mut C,
    start: (f32, f32),
    end: (f32, f32),
    color: C::Pixel,
    size: f64,
) where
    C: Canvas,
    C::Pixel: 'static,
{
    draw_line_segment_mut(canvas, start, end, color.clone());
    // from: https://stackoverflow.com/questions/10316180/how-to-calculate-the-coordinates-of-a-arrowhead-based-on-the-arrow
    if size > 1. {
        let h = size * 3.0f64.sqrt();
        let w = size;
        let start_position = Position::new(start.0 as f64, start.1 as f64);
        let end_position = Position::new(end.0 as f64, end.1 as f64);
        let u = vector_normalize(&vector_substract(&end_position, &start_position));
        let vw = vector_multiply(&Position::new(-u.y(), u.x()), w);
        let vv = vector_substract(&end_position, &vector_multiply(&u, h));
        let v1 = vector_add(&vv, &vw);
        let v2 = vector_substract(&vv, &vw);
        draw_line_segment_mut(canvas, end, (v1.x() as f32, v1.y() as f32), color.clone());
        draw_line_segment_mut(canvas, end, (v2.x() as f32, v2.y() as f32), color.clone());
        draw_line_segment_mut(
            canvas,
            (v1.x() as f32, v1.y() as f32),
            (v2.x() as f32, v2.y() as f32),
            color.clone(),
        );
    }
}
