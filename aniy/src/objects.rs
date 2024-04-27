//! Contains all builtin objects that can be rendered.
//! As well as the `Object` trait that all objects must implement,
//! and allows you to create custom objects.

use crate::Color;

/// The `Object` trait is implemented by all objects that can be rendered.
pub trait Object: Send + Sync {
    /// Renders the object into an SVG node.
    fn render(&self) -> (isize, Box<dyn svg::Node>);

    /// Get the bounding box of the object.
    ///
    /// You should not override the default implementation of this method
    /// as it uses the `render` method to get the bounding box.
    ///
    /// Unless you know what you are doing, and think you can do a more optimized version for your
    /// object.
    /// Which honestly wouldnt matter as this method is rarely called, and never in a hot path.
    /// so just use the default implementation.
    fn bounding_box(&self) -> resvg::usvg::Rect {
        let (_, node) = self.render();
        let doc = svg::Document::new().add(node);
        let node = crate::convert_to_resvg(doc.to_string());

        node.root().bounding_box()
    }
}

/// Represents a direction.
#[allow(missing_docs)] // Pretty self-explanatory
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// A polygon object.
#[derive(Clone)]
pub struct Polygon {
    /// The points of the polygon.
    ///
    /// The polygon is drawn by connecting the points in the order they are given.
    /// As well as the first and last point.
    pub points: Vec<(f32, f32)>,
    /// The fill color of the polygon.
    pub fill_color: Color,
    /// The outline color of the polygon.
    pub outline_color: Color,
    /// The stroke width of the polygon.
    pub stroke_width: f32,
    /// The z-index of the polygon.
    pub z_index: isize,
}

impl Default for Polygon {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            fill_color: Color::rgb(255, 255, 255),
            outline_color: Color::rgb(100, 100, 100),
            stroke_width: 10.0,
            z_index: 0,
        }
    }
}

impl Polygon {
    /// Creates a new polygon object.
    pub fn new(points: impl Into<Vec<(f32, f32)>>) -> Self {
        Self {
            points: points.into(),
            ..Default::default()
        }
    }

    /// Sets the z-index of the polygon.
    pub fn z_index(mut self, z_index: isize) -> Self {
        self.z_index = z_index;
        self
    }

    /// Adds a point to the polygon.
    pub fn add_point(mut self, x: f32, y: f32) -> Self {
        self.points.push((x, y));
        self
    }

    /// Shifts all the points of the polygon by `x` and `y`.
    ///
    /// Effectively moving the polygon.
    pub fn shift(mut self, x: f32, y: f32) -> Self {
        self.points = self
            .points
            .into_iter()
            .map(|(px, py)| (px + x, py + y))
            .collect();
        self
    }

    /// Sets the fill color of the polygon.
    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    /// Sets the outline color of the polygon.
    pub fn outline(mut self, color: Color) -> Self {
        self.outline_color = color;
        self
    }
}

impl Object for Polygon {
    fn render(&self) -> (isize, Box<dyn svg::Node>) {
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

        (self.z_index, Box::new(polygon))
    }
}

/// A text object.
#[derive(Clone)]
pub struct Text {
    /// The text to display.
    pub text: String,
    /// The x position of the anchor.
    pub x: f32,
    /// The y position of the anchor.
    pub y: f32,
    /// The font size of the text.
    pub font_size: f32,
    /// The color of the text.
    pub color: Color,
    /// The anchor of the text.
    /// This is where the x and y position of the text is relative to the actual text.
    ///
    /// see: https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/text-anchor
    pub anchor: String,
    /// The z-index of the text.
    pub z_index: isize,
}

impl Text {
    /// Creates a new text object.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            x: 0.0,
            y: 0.0,
            font_size: 100.0,
            color: Color::rgb(255, 255, 255),
            anchor: "middle".to_string(),
            z_index: 0,
        }
    }

    /// Sets the z-index of the text.
    pub fn z_index(mut self, z_index: isize) -> Self {
        self.z_index = z_index;
        self
    }

    /// Sets the anchor of the text.
    ///
    /// see: https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/text-anchor
    pub fn anchor(mut self, anchor: impl Into<String>) -> Self {
        self.anchor = anchor.into();
        self
    }

    /// Sets the position of the text.
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Sets the font size of the text.
    pub fn size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Sets the color of the text.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Move the text to appear besides another text object in  a certain direction.
    pub fn besides(mut self, other: &Text, dir: Direction) -> Self {
        let bounding_box = other.bounding_box();
        let (x, y) = match dir {
            Direction::Left => (bounding_box.left(), other.y),
            Direction::Right => (bounding_box.right(), other.y),
            Direction::Up => (other.x, bounding_box.top()),
            Direction::Down => (other.x, bounding_box.bottom()),
        };
        self.x = x;
        self.y = y;

        self
    }

    /// Move the text by `x` and `y`.
    pub fn shift(mut self, x: f32, y: f32) -> Self {
        self.x += x;
        self.y += y;
        self
    }

    /// Gets the duration it would take to type out the text with a specific wpm.
    pub fn wpm(&self, wpm: f32) -> f32 {
        /// The average word length in characters.
        /// Because ofc the typing speed will be constant, but it is given in words per minute.
        ///
        /// (wpm is really a bad way to measure typing speed, but it is what it is)
        const AVG_WORD_LENGTH: f32 = 5.0;

        self.text.len() as f32 / AVG_WORD_LENGTH / wpm * 60.0
    }
}

impl Object for Text {
    fn render(&self) -> (isize, Box<dyn svg::Node>) {
        let mut text =
            svg::node::element::Text::new(self.text.clone());

        text = text
            .set("x", self.x)
            .set("y", self.y)
            .set("font-size", self.font_size)
            .set("fill", self.color.as_css().as_ref())
            .set("fill-opacity", self.color.3 as f32 / 255.0)
            .set("text-anchor", self.anchor.as_str());

        (self.z_index, Box::new(text))
    }
}

/// A raw SVG object.
pub struct RawSvg(String);

impl RawSvg {
    /// Creates a new raw SVG object.
    pub fn new(svg: impl Into<String>) -> Self {
        Self(svg.into())
    }
}

impl Object for RawSvg {
    fn render(&self) -> (isize, Box<dyn svg::Node>) {
        (0, Box::new(svg::node::Blob::new(&self.0)))
    }
}

/// Render a math expression using mathjax.
#[derive(Clone)]
pub struct Math {
    /// The math expression to render.
    pub text: String,
    /// The color of the math expression.
    pub color: Color,
    /// The size of the math expression.
    pub size: f32,
    /// The x position of the math expression.
    pub x: f32,
    /// The y position of the math expression.
    pub y: f32,
    /// The z-index of the math expression.
    pub z_index: isize,
}

impl Math {
    /// Creates a new math object.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::rgb(255, 255, 255),
            size: 10.0,
            x: 0.0,
            y: 0.0,
            z_index: 0,
        }
        .center_on(0.0, 0.0)
    }

    /// Sets the position of the math expression.
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Sets the size of the math expression.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the color of the math expression.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the z-index of the math expression.
    pub fn z_index(mut self, z_index: isize) -> Self {
        self.z_index = z_index;
        self
    }

    /// Centers the math expression on a point.
    pub fn center_on(mut self, x: f32, y: f32) -> Self {
        let bounding_box = self.bounding_box();
        self.x = x - bounding_box.width() / 2.0;
        self.y = y - bounding_box.height() / 2.0;
        self
    }
}

impl Object for Math {
    fn render(&self) -> (isize, Box<dyn svg::Node>) {
        let renderer = mathjax::MathJax::new().unwrap();
        let mut result = renderer.render(&self.text).unwrap();
        result.set_color(self.color.as_css().as_ref());
        let svg = result.into_raw();

        let transform = format!(
            "translate({}, {}) scale({})",
            self.x, self.y, self.size
        );
        let svg = format!(
            r#"
            <g transform="{}">{}</g>
            "#,
            transform, svg
        );

        (self.z_index, Box::new(svg::node::Blob::new(svg)))
    }
}
