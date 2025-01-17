use super::vector::*;
use super::element::*;
use super::properties::*;
use super::control_point::*;
use super::transformed_vector::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::brush::*;
use super::super::motion::*;

use flo_canvas::*;

use itertools::*;
use std::sync::*;
use std::time::Duration;

///
/// Element representing a brush stroke
///
#[derive(Clone, Debug)]
pub struct BrushElement {
    /// The ID of this element
    id: ElementId,

    /// The path taken by this brush stroke
    points: Arc<Vec<BrushPoint>>,
}

impl BrushElement {
    ///
    /// Begins a new brush stroke at a particular position
    ///
    pub fn new(id: ElementId, points: Arc<Vec<BrushPoint>>) -> BrushElement {
        BrushElement {
            id:                 id,
            points:             points
        }
    }

    ///
    /// Retrieves the points in this brush element
    ///
    pub fn points(&self) -> Arc<Vec<BrushPoint>> {
        Arc::clone(&self.points)
    }

    ///
    /// Moves this brush stroke so that it fits within a particular bounding box
    /// (when rendered with a particular set of properties)
    ///
    pub fn move_to(&mut self, new_bounds: Rect, properties: &VectorProperties) {
        // Scale using the existing bounds
        let existing_bounds = self.to_path(properties)
            .map(|paths| paths.into_iter()
                .map(|path| Rect::from(&path))
                .fold(Rect::empty(), |a, b| a.union(b)))
            .unwrap_or(Rect::empty());

        let (current_w, current_h)  = (existing_bounds.x2-existing_bounds.x1, existing_bounds.y2-existing_bounds.y1);
        let (new_w, new_h)          = (new_bounds.x2-new_bounds.x1, new_bounds.y2-new_bounds.y1);
        let (scale_x, scale_y)      = (new_w/current_w, new_h/current_h);

        // Functions to transform the points in this brush stroke
        let transform       = |(x, y)| {
            ((x - existing_bounds.x1)*scale_x + new_bounds.x1,
             (y - existing_bounds.y1)*scale_y + new_bounds.y1)
        };

        let transform_point = |point: &BrushPoint| {
            BrushPoint {
                position:   transform(point.position),
                cp1:        transform(point.cp1),
                cp2:        transform(point.cp2),
                width:      point.width
            }
        };

        // Perform the transformation itself
        let new_points      = self.points.iter()
            .map(|old_point| transform_point(old_point))
            .collect();
        self.points = Arc::new(new_points);
    }
}

impl VectorElement for BrushElement {
    ///
    /// The ID of this vector element
    ///
    fn id(&self) -> ElementId {
        self.id
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, _when: Duration) {
        gc.draw_list(properties.brush.render_brush(&properties.brush_properties, &self.points))
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>> {
        Some(vec![Path::from_drawing(properties.brush.render_brush(&properties.brush_properties, &self.points))])
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector {
        let transformed_points = motion.transform_brush_points(when, self.points.iter()).collect();

        let transformed = Vector::BrushStroke(BrushElement {
            id:     self.id,
            points: Arc::new(transformed_points)
        });

        Vector::Transformed(TransformedVector::new(Vector::BrushStroke(self.clone()), transformed))
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self) -> Vec<ControlPoint> {
        self.points.iter()
            .flat_map(|brush_point| {
                vec![
                    ControlPoint::BezierControlPoint(brush_point.cp1.0, brush_point.cp1.1),
                    ControlPoint::BezierControlPoint(brush_point.cp2.0, brush_point.cp2.1),
                    ControlPoint::BezierPoint(brush_point.position.0, brush_point.position.1)
                ]
            })
            .skip(2)
            .collect()
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector {
        // The widths are kept the same as they are in this element
        let widths = self.points.iter().map(|point| point.width);

        // The first element still has two control points, but we only actually care about its position. Generate two fake control points here.
        let initial_pos             = new_positions[0];
        let initial_control_points  = vec![initial_pos, initial_pos];

        // Turn the set of control point positions into a set of control points
        // Using more elements than we already have will just clip the result to the same number of points
        // Using fewer elements will cause the result to have fewer elements
        // Neither of these behaviours is a good way to change the number of points in the result
        let brush_elements          = initial_control_points.into_iter().chain(new_positions.into_iter())
            .tuples()
            .zip(widths)
            .map(|((cp1, cp2, pos), width)| BrushPoint {
                position:   pos,
                cp1:        cp1,
                cp2:        cp2,
                width:      width
            });

        // Create a new brush element
        Vector::BrushStroke(BrushElement {
            id:     self.id,
            points: Arc::new(brush_elements.collect())
        })
    }
}

impl Into<Vector> for BrushElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushStroke(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fetch_control_points_for_brush_stroke() {
        let points = vec![
            BrushPoint {
                position: (1.0, 2.0),
                cp1: (3.0, 4.0),
                cp2: (5.0, 6.0),
                width: 0.5
            },

            BrushPoint {
                position: (7.0, 8.0),
                cp1: (9.0, 10.0),
                cp2: (11.0, 12.0),
                width: 0.6
            },

            BrushPoint {
                position: (13.0, 14.0),
                cp1: (15.0, 16.0),
                cp2: (17.0, 18.0),
                width: 0.7
            },
        ];
        let element = BrushElement::new(ElementId::Assigned(5), Arc::new(points));

        let control_points = element.control_points();

        assert!(control_points[0] == ControlPoint::BezierPoint(1.0, 2.0));
        assert!(control_points[1] == ControlPoint::BezierControlPoint(9.0, 10.0));
        assert!(control_points[2] == ControlPoint::BezierControlPoint(11.0, 12.0));
        assert!(control_points[3] == ControlPoint::BezierPoint(7.0, 8.0));
        assert!(control_points[4] == ControlPoint::BezierControlPoint(15.0, 16.0));
        assert!(control_points[5] == ControlPoint::BezierControlPoint(17.0, 18.0));
        assert!(control_points[6] == ControlPoint::BezierPoint(13.0, 14.0));
        assert!(control_points.len() == 7);
    }

    #[test]
    fn update_control_points_for_brush_stroke() {
        let points = vec![
            BrushPoint {
                position: (1.0, 2.0),
                cp1: (3.0, 4.0),
                cp2: (5.0, 6.0),
                width: 0.5
            },

            BrushPoint {
                position: (7.0, 8.0),
                cp1: (9.0, 10.0),
                cp2: (11.0, 12.0),
                width: 0.6
            },

            BrushPoint {
                position: (13.0, 14.0),
                cp1: (15.0, 16.0),
                cp2: (17.0, 18.0),
                width: 0.7
            },
        ];
        let element = BrushElement::new(ElementId::Assigned(5), Arc::new(points));
        let updated = element.with_adjusted_control_points(vec![
            (1.1, 1.2),
            (2.1, 2.2),
            (3.1, 3.2),
            (4.1, 4.2),
            (5.1, 5.2),
            (6.1, 6.2),
            (7.1, 7.2)
        ]);

        let control_points = updated.control_points();

        assert!(control_points[0] == ControlPoint::BezierPoint(1.1, 1.2));
        assert!(control_points[1] == ControlPoint::BezierControlPoint(2.1, 2.2));
        assert!(control_points[2] == ControlPoint::BezierControlPoint(3.1, 3.2));
        assert!(control_points[3] == ControlPoint::BezierPoint(4.1, 4.2));
        assert!(control_points[4] == ControlPoint::BezierControlPoint(5.1, 5.2));
        assert!(control_points[5] == ControlPoint::BezierControlPoint(6.1, 6.2));
        assert!(control_points[6] == ControlPoint::BezierPoint(7.1, 7.2));
        assert!(control_points.len() == 7);

        if let Vector::BrushStroke(updated) = updated {
            assert!(updated.points[0].width == 0.5);
            assert!(updated.points[1].width == 0.6);
            assert!(updated.points[2].width == 0.7);
        }
    }
}
