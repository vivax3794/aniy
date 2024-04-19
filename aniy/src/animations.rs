use std::sync::Arc;

use crate::objects::{self, Object};

pub trait Animation: Send + Sync {
    fn animate(&self, progress: f32) -> Box<dyn svg::Node>;
}

#[derive(Clone)]
pub struct AnimationContainer {
    pub animation: Arc<dyn Animation>,
    pub start: f32,
    pub end: f32,
}

impl AnimationContainer {
    pub fn new(animation: Arc<dyn Animation>) -> Self {
        Self {
            animation,
            start: 0.0,
            end: 1.0,
        }
    }

    pub(crate) fn animate(&self, time: f32) -> Box<dyn svg::Node> {
        let progress = (time - self.start) / (self.end - self.start);
        if !(0.0..=1.0).contains(&progress) {
            log::warn!("Progress out of bounds: {}", progress);
        }
        self.animation.animate(progress)
    }

    pub fn duration(mut self, duration: f32) -> Self {
        self.end = self.start + duration;
        self
    }
    pub fn delay(mut self, delay: f32) -> Self {
        self.start += delay;
        self.end += delay;
        self
    }
    pub fn after(mut self, other: &AnimationContainer) -> Self {
        let duration = self.end - self.start;
        self.start = other.end;
        self.end = self.start + duration;
        self
    }

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

pub struct AnimatedObject {
    pub object: Arc<dyn Object>,
    pub enter: AnimationContainer,
    pub exit: AnimationContainer,
}

impl AnimatedObject {
    pub fn lifetime(mut self, duration: f32) -> Self {
        let exit_duration = self.exit.end - self.exit.start;
        self.exit.start = self.enter.end + duration;
        self.exit = self.exit.duration(exit_duration);
        self
    }
}

pub struct NoAnimation;

impl Animation for NoAnimation {
    fn animate(&self, _progress: f32) -> Box<dyn svg::Node> {
        Box::new(svg::node::element::Group::new())
    }
}

pub struct ReverseAnimation {
    pub animation: Arc<dyn Animation>,
}

impl Animation for ReverseAnimation {
    fn animate(&self, progress: f32) -> Box<dyn svg::Node> {
        self.animation.animate(1.0 - progress)
    }
}

pub struct FadeAnimation(pub Arc<dyn Object>);

impl Animation for FadeAnimation {
    fn animate(&self, progress: f32) -> Box<dyn svg::Node> {
        let group = svg::node::element::Group::new();
        let group =
            group.add(self.0.render()).set("opacity", progress);

        Box::new(group)
    }
}

pub struct PolygonAnimation(pub Arc<objects::Polygon>);

impl Animation for PolygonAnimation {
    fn animate(&self, progress: f32) -> Box<dyn svg::Node> {
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
        let x = start.0 + (end.0 - start.0) * segment_progress as f64;
        let y = start.1 + (end.1 - start.1) * segment_progress as f64;

        points.push((x, y));
        polygon.points = points.clone();
        let outline_color = polygon.outline_color;
        polygon.outline_color = None;
        let polygon_render = polygon.render();

        let mut line = svg::node::element::Polyline::new()
            .set("points", points)
            .set("fill", "none")
            .set("stroke-width", polygon.stroke_width);
        if let Some(color) = outline_color {
            line = line.set("stroke", color.to_css().as_ref());
        }

        let group = svg::node::element::Group::new()
            .add(polygon_render)
            .add(line);
        Box::new(group)
    }
}
