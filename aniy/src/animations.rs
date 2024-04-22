//! Contains the animation system for the library.
//! As well as animations for the builtin objects, and objects in general.

use std::sync::Arc;

use crate::{
    objects::{self, Object},
    Color,
};

/// The `Animation` trait is implemented by all animations.
pub trait Animation: Send + Sync {
    /// Given a progress value between 0.0 and 1.0, returns the z-index and the SVG node.
    fn animate(&self, progress: f32) -> (isize, Box<dyn svg::Node>);
}

/// A wrapper around a animation to provide duration, delay, and other features.
#[derive(Clone)]
pub struct AnimationContainer {
    /// The animation to be wrapped.
    pub animation: Arc<dyn Animation>,
    /// The start time of the animation in seconds.
    pub start: f32,
    /// The end time of the animation in seconds.
    pub end: f32,
}

impl AnimationContainer {
    /// Creates a new `AnimationContainer` with the given animation.
    ///
    /// Default duration is 1 second starting at 0 seconds.
    pub fn new(animation: Arc<dyn Animation>) -> Self {
        Self {
            animation,
            start: 0.0,
            end: 1.0,
        }
    }

    /// Animate the animation at the given time by calculating the progress.
    pub(crate) fn animate(
        &self,
        time: f32,
    ) -> (isize, Box<dyn svg::Node>) {
        let progress = (time - self.start) / (self.end - self.start);
        let progress = progress.clamp(0.0, 1.0);

        self.animation.animate(progress)
    }

    /// Set the end time as to make the duration of the animation the given duration.
    pub fn duration(mut self, duration: f32) -> Self {
        self.end = self.start + duration;
        self
    }

    /// Set the start time as to make the duration of the animation the given duration.
    pub fn duration_keep_end(mut self, duration: f32) -> Self {
        self.start = self.end - duration;
        self
    }

    /// Shift the start and end time by the given delay.
    pub fn delay(mut self, delay: f32) -> Self {
        self.start += delay;
        self.end += delay;
        self
    }

    /// Set the start time to the end time of the given animation.
    /// Preserving the duration of the animation.
    pub fn after(mut self, other: &AnimationContainer) -> Self {
        let duration = self.end - self.start;
        self.start = other.end;
        self.end = self.start + duration;
        self
    }

    /// Set the start time to the start time of the given animation.
    pub fn start_with(mut self, other: &AnimationContainer) -> Self {
        self.start = other.start;
        self
    }

    /// Set the end time to the end time of the given animation.
    pub fn end_with(mut self, other: &AnimationContainer) -> Self {
        self.end = other.end;
        self
    }

    /// Synchronize the start and end time with the given animation.
    pub fn synchronize(mut self, other: &AnimationContainer) -> Self {
        self.start = other.start;
        self.end = other.end;
        self
    }

    /// Reverse the animation.
    pub fn reverse(self) -> Self {
        Self {
            animation: Arc::new(ReverseAnimation {
                animation: self.animation,
            }),
            start: self.start,
            end: self.end,
        }
    }
}

/// Holds an object and the enter and exit animations for it.
///
/// After the enter animation is done, the object will be inserted into the scene.
/// And at the start of the exit animation, the object will be removed from the scene.
pub struct AnimatedObject {
    /// The object to render between the enter and exit animations.
    pub object: Arc<dyn Object>,
    /// The enter animation.
    pub enter: AnimationContainer,
    /// The exit animation.
    pub exit: AnimationContainer,
}

impl AnimatedObject {
    /// Move the start time of the end animation so it is `duration` seconds after the end of the enter animation.
    pub fn lifetime(mut self, duration: f32) -> Self {
        let exit_duration = self.exit.end - self.exit.start;
        self.exit.start = self.enter.end + duration;
        self.exit = self.exit.duration(exit_duration);
        self
    }
}

/// An animation that does nothing.
///
/// Useful when you just want the object to appear without any animation.
pub struct NoAnimation;

impl Animation for NoAnimation {
    fn animate(&self, _progress: f32) -> (isize, Box<dyn svg::Node>) {
        (0, Box::new(svg::node::element::Group::new()))
    }
}

/// An animation that reverses the given animation.
pub struct ReverseAnimation {
    /// The animation to reverse.
    pub animation: Arc<dyn Animation>,
}

impl Animation for ReverseAnimation {
    fn animate(&self, progress: f32) -> (isize, Box<dyn svg::Node>) {
        self.animation.animate(1.0 - progress)
    }
}

/// An animation that fades in the given object.
///
/// Works on any object.
pub struct FadeAnimation(isize, Box<dyn svg::Node>);

impl FadeAnimation {
    /// Create a new `FadeAnimation` from the given object.
    /// By pre-rendering the object.
    pub fn new(object: Arc<dyn Object>) -> Self {
        let (z, node) = object.render();
        Self(z, node)
    }
}

impl Animation for FadeAnimation {
    fn animate(&self, progress: f32) -> (isize, Box<dyn svg::Node>) {
        let group = svg::node::element::Group::new();
        let group =
            group.add(self.1.clone()).set("opacity", progress);

        (self.0, Box::new(group))
    }
}

/// An animation that draws in a polygon from the first point to the last.
pub struct PolygonDraw(pub Arc<objects::Polygon>);

impl Animation for PolygonDraw {
    fn animate(&self, progress: f32) -> (isize, Box<dyn svg::Node>) {
        let mut polygon = (*self.0).clone();

        let done_amount =
            (polygon.points.len() as f32 * progress).floor() as usize;
        if done_amount == polygon.points.len() {
            return polygon.render();
        }

        let mut points = Vec::with_capacity(done_amount);

        for point in &polygon.points[..done_amount + 1] {
            points.push(*point);
        }

        let start = polygon.points[done_amount];
        let end =
            polygon.points[(done_amount + 1) % polygon.points.len()];

        let segment_progress = progress * polygon.points.len() as f32
            - done_amount as f32;
        let x = start.0 + (end.0 - start.0) * segment_progress;
        let y = start.1 + (end.1 - start.1) * segment_progress;

        points.push((x, y));
        polygon.points = points.clone();
        let outline_color = polygon.outline_color;
        polygon.outline_color = Color(0, 0, 0, 0);
        let (z, polygon_render) = polygon.render();

        let mut line = svg::node::element::Polyline::new()
            .set("points", points)
            .set("fill", "none")
            .set("stroke-width", polygon.stroke_width);
        line = line.set("stroke", outline_color.as_css().as_ref());

        let group = svg::node::element::Group::new()
            .add(polygon_render)
            .add(line);
        (z, Box::new(group))
    }
}

/// An animation that morphs a polygon from one shape to another.
pub struct PolygonMorph {
    /// The starting polygon.
    start_polygon: Arc<objects::Polygon>,
    /// The ending polygon.
    end_polygon: Arc<objects::Polygon>,
    /// The starting points of the polygon potentially with missing points inserted.
    start_points: Vec<(f32, f32)>,
    /// The ending points of the polygon potentially with missing points inserted.
    end_points: Vec<(f32, f32)>,
}

impl PolygonMorph {
    /// Create a new `PolygonMorph` from the given polygons.
    pub fn new(
        start_polygon: Arc<objects::Polygon>,
        end_polygon: Arc<objects::Polygon>,
    ) -> Self {
        let mut start_points = start_polygon.points.clone();
        let mut end_points = end_polygon.points.clone();

        match start_points.len().cmp(&end_points.len()) {
            std::cmp::Ordering::Less => {
                create_missing_points(
                    &mut start_points,
                    &mut end_points,
                );
            }
            std::cmp::Ordering::Greater => {
                create_missing_points(
                    &mut end_points,
                    &mut start_points,
                );
            }
            std::cmp::Ordering::Equal => {}
        }

        Self {
            start_polygon,
            end_polygon,
            start_points,
            end_points,
        }
    }
}

impl Animation for PolygonMorph {
    fn animate(&self, progress: f32) -> (isize, Box<dyn svg::Node>) {
        let mut points = Vec::with_capacity(self.start_points.len());

        for (start, end) in
            self.start_points.iter().zip(self.end_points.iter())
        {
            let x = start.0 + (end.0 - start.0) * progress;
            let y = start.1 + (end.1 - start.1) * progress;
            points.push((x, y));
        }

        let fill_color = self
            .start_polygon
            .fill_color
            .morph(&self.end_polygon.fill_color, progress);
        let outline_color = self
            .start_polygon
            .outline_color
            .morph(&self.end_polygon.outline_color, progress);

        let polygon = objects::Polygon::new(points)
            .fill(fill_color)
            .outline(outline_color);

        polygon.render()
    }
}

/// A point
type Point = (f32, f32);

/// Create points on the shorter polygon such that animating from the shorter to the longer polygon is smooth.
fn create_missing_points(
    short: &mut Vec<Point>,
    longer: &mut Vec<Point>,
) {
    let short_first = top_left(short);
    let short_trans =
        translate_points(short, -short_first.0, -short_first.1);

    let long_first = top_left(longer);
    let mut long_trans =
        translate_points(longer, -long_first.0, -long_first.1)
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();

    let mut static_points = vec![];
    for point in &short_trans {
        let long_trans_points =
            long_trans.iter().map(|(_, p)| *p).collect::<Vec<_>>();
        let (i, _) = closest_point(point, &long_trans_points);
        static_points.push((long_trans[i].0, *point));
        long_trans.remove(i);
    }

    let mut segments = vec![vec![]; short_trans.len()];

    let point_pairs = (0..short_trans.len())
        .map(|i| {
            (short_trans[i], short_trans[(i + 1) % short_trans.len()])
        })
        .collect::<Vec<_>>();
    for (li, point) in long_trans {
        let (i, (closet, _)) = point_pairs
            .iter()
            .map(|(a, b)| cast_to_line(*a, *b, point))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        segments[i].push((li, closet));
    }

    let mut points = vec![];
    for (i, segment) in segments.iter().enumerate() {
        points.push(static_points[i]);
        points.extend(segment);
    }
    *longer =
        points.iter().map(|(i, _)| longer[*i]).collect::<Vec<_>>();
    *short = translate_points(
        &points.into_iter().map(|(_, a)| a).collect::<Vec<_>>(),
        short_first.0,
        short_first.1,
    );
}

/// Find the closest point in the segment to the given point.
fn closest_point(point: &Point, segment: &[Point]) -> (usize, Point) {
    segment
        .iter()
        .copied()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            distance(*point, *a)
                .partial_cmp(&distance(*point, *b))
                .unwrap()
        })
        .unwrap()
}

/// Find the point on the line that is closest to the given point.
fn cast_to_line(
    line_a: Point,
    line_b: Point,
    point: Point,
) -> (Point, f32) {
    let line_b = (line_b.0 - line_a.0, line_b.1 - line_a.1);
    let point = (point.0 - line_a.0, point.1 - line_a.1);

    let line_angle = line_b.1.atan2(line_b.0);
    let rotated_point = rotate_point(point, -line_angle);

    let fallen_point = (
        rotated_point.0.clamp(0.0, distance((0.0, 0.0), line_b)),
        0.0,
    );
    let rotated_fallen_point = rotate_point(fallen_point, line_angle);
    let distance = distance(point, rotated_fallen_point);
    let fallen_point = (
        rotated_fallen_point.0 + line_a.0,
        rotated_fallen_point.1 + line_a.1,
    );

    (fallen_point, distance)
}

/// Rotate a point by the given angle.
fn rotate_point(point: Point, angle: f32) -> Point {
    (
        point.0 * angle.cos() - point.1 * angle.sin(),
        point.0 * angle.sin() + point.1 * angle.cos(),
    )
}

/// Calculate the distance between two points.
fn distance(a: Point, b: Point) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// Translate a point by the given x and y.
fn translate_point(point: Point, x: f32, y: f32) -> Point {
    (point.0 + x, point.1 + y)
}

/// Translate all the points by the given x and y.
fn translate_points(points: &[Point], x: f32, y: f32) -> Vec<Point> {
    points
        .iter()
        .map(|point| translate_point(*point, x, y))
        .collect()
}

/// Find the top left point of the given points.
fn top_left(points: &[Point]) -> Point {
    points
        .iter()
        .fold((f32::INFINITY, f32::INFINITY), |acc, p| {
            (acc.0.min(p.0), acc.1.min(p.1))
        })
}

/// An animation that types out the text.
pub struct TextType(pub Arc<objects::Text>);

impl Animation for TextType {
    fn animate(&self, progress: f32) -> (isize, Box<dyn svg::Node>) {
        let mut text = (*self.0).clone();
        let chars_count = text.text.chars().count();
        let chars_done =
            (chars_count as f32 * progress).floor() as usize;
        let mut chars =
            text.text.chars().take(chars_done).collect::<String>();

        if chars_done != chars_count {
            chars.push('_');
        }

        text.text = chars;
        text.render()
    }
}

/// An animation that writes out the text by drawing the path of the text.
/// Similar to `PolygonDraw` but for each segment of the characters.
///
/// Honestly this doesnt look that good, but it is kind of cool.
/// Would love to get something more like manim's write animation,
/// but not clue how to do that in SVG.
pub struct TextWrite(Vec<String>, Color);

impl TextWrite {
    /// Create a new `TextWrite` from the given text.
    pub fn new(text: &objects::Text) -> Self {
        let path_segments =
            calculate_path_segements_from_text(text.render().1);
        Self(path_segments, text.color)
    }
}

impl Animation for TextWrite {
    fn animate(&self, progress: f32) -> (isize, Box<dyn svg::Node>) {
        let amount_segments =
            (self.0.len() as f32 * progress).floor() as usize;
        let path = self.0[..amount_segments].join(" ");

        let path = svg::node::element::Path::new()
            .set("d", path)
            .set("fill", self.1.as_css().as_ref())
            .set("stroke-width", 5);

        (0, Box::new(path))
    }
}

/// Calculate the path segments from the text node.
fn calculate_path_segements_from_text(
    node: Box<dyn svg::Node>,
) -> Vec<String> {
    let doc = svg::Document::new().add(node);
    let node = crate::convert_to_resvg(doc.to_string());
    let node = node.root().children()[0].clone();

    let resvg::usvg::Node::Text(node) = node else {
        panic!("Expected text node");
    };
    let paths = node.flattened();
    let path = paths.children()[0].clone();

    let resvg::usvg::Node::Path(path) = path else {
        panic!("Expected path node");
    };

    let segments = path.data().segments().collect::<Vec<_>>();

    use resvg::tiny_skia::PathSegment;
    let mut string_segments = vec![];
    for segment in segments.iter() {
        string_segments.push(match segment {
            PathSegment::MoveTo(p) => {
                format!("M {} {} ", p.x, p.y)
            }
            PathSegment::LineTo(p) => {
                format!("L {} {} ", p.x, p.y)
            }
            PathSegment::QuadTo(p0, p1) => {
                format!("Q {} {} {} {} ", p0.x, p0.y, p1.x, p1.y)
            }
            PathSegment::CubicTo(p0, p1, p2) => format!(
                "C {} {} {} {} {} {} ",
                p0.x, p0.y, p1.x, p1.y, p2.x, p2.y,
            ),
            PathSegment::Close => "Z ".to_string(),
        });
    }
    string_segments
}
