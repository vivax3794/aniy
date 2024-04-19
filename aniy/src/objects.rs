use crate::Color;

pub trait Object: Send + Sync {
    fn render(&self) -> Box<dyn svg::Node>;
}

#[derive(Clone)]
pub struct Polygon {
    pub points: Vec<(f64, f64)>,
    pub fill_color: Color,
    pub outline_color: Color,
    pub stroke_width: f64,
}

impl Default for Polygon {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            fill_color: Color::rgb(255, 255, 255),
            outline_color: Color::rgb(100, 100, 100),
            stroke_width: 10.0,
        }
    }
}

impl Polygon {
    pub fn new(points: impl Into<Vec<(f64, f64)>>) -> Self {
        Self {
            points: points.into(),
            ..Default::default()
        }
    }

    pub fn add_point(mut self, x: f64, y: f64) -> Self {
        self.points.push((x, y));
        self
    }

    pub fn shift(mut self, x: f64, y: f64) -> Self {
        self.points = self
            .points
            .into_iter()
            .map(|(px, py)| (px + x, py + y))
            .collect();
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn outline(mut self, color: Color) -> Self {
        self.outline_color = color;
        self
    }
}

impl Object for Polygon {
    fn render(&self) -> Box<dyn svg::Node> {
        let mut polygon = svg::node::element::Polygon::new();

        polygon = polygon
            .set(
                "points",
                self.points
                    .iter()
                    .map(|(x, y)| format!("{},{}", x, y))
                    .collect::<Vec<_>>()
                    .join(" "),
            )
            .set("stroke-width", self.stroke_width);

        polygon =
            polygon.set("fill", self.fill_color.as_css().as_ref());
        polygon = polygon
            .set("stroke", self.outline_color.as_css().as_ref());

        Box::new(polygon)
    }
}
