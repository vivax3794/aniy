//! Manim inspired animation library for Rust built on SVG.
//!
//! # Stability
//! This is a personal project and is not intended for production use.
//! The API is not stable and may change at any time.
//! I am making this for my own personal use and learning.
//! And do not have any plans to maintain this library in the long term.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use indicatif::ParallelProgressIterator;
use indicatif::ProgressIterator;
use rayon::prelude::*;
use std::sync::Arc;

use video_rs::Time;

pub mod animations;
pub mod objects;

/// A color with red, green, blue and alpha components.
#[derive(Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

impl Color {
    /// Darkens the color by a certain amount.
    pub fn darken(&self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        let r = (self.0 as f32 * amount) as u8;
        let g = (self.1 as f32 * amount) as u8;
        let b = (self.2 as f32 * amount) as u8;
        Self(r, g, b, self.3)
    }

    /// Linearly interpolates between two colors.
    fn morph(&self, other: &Self, progress: f32) -> Self {
        let r = (self.0 as f32
            + (other.0 as f32 - self.0 as f32) * progress)
            as u8;
        let g = (self.1 as f32
            + (other.1 as f32 - self.1 as f32) * progress)
            as u8;
        let b = (self.2 as f32
            + (other.2 as f32 - self.2 as f32) * progress)
            as u8;
        let a = (self.3 as f32
            + (other.3 as f32 - self.3 as f32) * progress)
            as u8;
        Self(r, g, b, a)
    }

    /// Creates a new color with the given red, green and blue components.
    ///
    /// The alpha component is set to 255.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b, 255)
    }

    /// Converts the color to a CSS color string.
    fn as_css(&self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            self.0, self.1, self.2, self.3
        )
    }
}

/// A frame holds all the info needed to render that frame.
#[derive(Clone)]
struct Frame {
    /// The timestamp of the frame in seconds.
    time: f32,
    /// The pre-rendered objects to be rendered in the frame.
    objects: Vec<(isize, Box<dyn svg::Node>)>,
    /// The animations to be calculated and rendered in the frame.
    animations: Vec<Arc<animations::AnimationContainer>>,
}

/// Holds all objects and animations in the video.
///
/// The length of the video will be based on the end time of the last animation.
#[derive(Default)]
pub struct Timeline {
    /// Static objects to be rendered in the video.
    objects: Vec<(isize, Box<dyn svg::Node>)>,
    /// Animated objects to be rendered in the video.
    ///
    /// These have a enter and exit animation.
    animations: Vec<Arc<animations::AnimatedObject>>,
}

impl Timeline {
    /// Add a static object to the timeline.
    ///
    /// Note: if no animations are added, then the video duration will be 0s.
    pub fn add_object(
        &mut self,
        object: Arc<dyn objects::Object>,
    ) -> &mut Self {
        self.objects.push(object.render());
        self
    }

    /// Add an animation to the timeline.
    ///
    /// Note: if you have a `Arc<AnimatedObject>`, use `add_animation_arc`.
    pub fn add_animation(
        &mut self,
        animated_object: animations::AnimatedObject,
    ) -> &mut Self {
        self.animations.push(Arc::new(animated_object));
        self
    }

    /// Add an animation to the timeline.
    pub fn add_animation_arc(
        &mut self,
        animated_object: Arc<animations::AnimatedObject>,
    ) -> &mut Self {
        self.animations.push(animated_object);
        self
    }

    /// Calculate all the frames in the video.
    ///
    /// This is done by calculating the animations and objects present on each frame.
    fn calc_frames(&self, fps: usize) -> Vec<Frame> {
        let end_time = self
            .animations
            .iter()
            .map(|animated_object| animated_object.exit.end)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let frame_count =
            (end_time * fps as f32).ceil() as usize + 10;

        log::info!(
            "Video will be {} frames ({:.2}s)",
            frame_count,
            end_time
        );

        let frame_duration = 1.0 / fps as f32;
        let mut frames = Vec::with_capacity(frame_count);

        log::info!("Creating frame objects");
        for frame_index in 0..frame_count {
            let time = frame_index as f32 * frame_duration;
            let objects = self.objects.clone();
            frames.push(Frame {
                time,
                objects,
                animations: Vec::new(),
            });
        }

        log::info!("Resolving {} animations", self.animations.len());
        for animated_object in &self.animations {
            let enter_animation =
                Arc::new(animated_object.enter.clone());
            for index in frame_range(
                animated_object.enter.start,
                animated_object.enter.end,
                fps,
            ) {
                frames[index]
                    .animations
                    .push(enter_animation.clone());
            }

            let exit_animation =
                Arc::new(animated_object.exit.clone());
            for index in frame_range(
                animated_object.exit.start,
                animated_object.exit.end,
                fps,
            ) {
                frames[index].animations.push(exit_animation.clone());
            }

            let object = animated_object.object.render();
            for index in frame_range(
                animated_object.enter.end,
                animated_object.exit.start,
                fps,
            ) {
                frames[index].objects.push(object.clone());
            }
        }

        frames
    }
}

/// Calculates and returns a iterator of all frame indexes between the start and end time.
fn frame_range(
    start: f32,
    end: f32,
    fps: usize,
) -> impl Iterator<Item = usize> {
    let frame_duration = 1.0 / fps as f32;
    let start_frame = (start / frame_duration).floor() as usize;
    let end_frame = (end / frame_duration).ceil() as usize;
    start_frame..end_frame
}

/// The core renderer for the library.
pub struct Renderer {
    /// The width of the video.
    width: usize,
    /// The height of the video.
    height: usize,
    /// The frames per second of the video.
    fps: u32,
    /// The timeline of the video.
    timeline: Timeline,
}

impl Renderer {
    /// Creates a new renderer with the given width and height.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            fps: 60,
            timeline: Default::default(),
        }
    }

    /// Sets the frames per second of the video.
    ///
    /// Defaults to 60fps.
    pub fn set_fps(&mut self, fps: u32) -> &mut Self {
        self.fps = fps;
        self
    }

    /// Gets a reference to the timeline, which is used to add objects and animations.
    pub fn timeline(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    /// Render the video and return the output location.
    pub fn render(self) -> RenderingResult {
        log::info!("Initing rendering runtime");

        let output_location = std::path::Path::new("output.mp4");

        video_rs::init().unwrap();
        let settings =
            video_rs::encode::Settings::preset_h264_yuv420p(
                self.width,
                self.height,
                false,
            );
        let mut encoder =
            video_rs::encode::Encoder::new(output_location, settings)
                .unwrap();

        let mut video_position = Time::zero();
        let frame_duration = Time::from_secs(1.0 / self.fps as f32);

        log::info!("Calculating timeline/frames");
        let frames = self.timeline.calc_frames(self.fps as usize);

        log::info!("Rendering frames");
        let frames_count = frames.len();
        let frames = frames
            // .into_iter()
            .into_par_iter()
            .progress_count(frames_count as u64)
            .panic_fuse()
            .map(|frame| {
                let doc = self.render_frame(frame);
                self.render_svg(doc)
            })
            .collect::<Vec<_>>();

        log::info!("Encoding frames");
        for frame in frames.into_iter().progress() {
            encoder.encode(&frame, &video_position).unwrap();
            video_position =
                video_position.aligned_with(&frame_duration).add();
        }

        log::info!("Finishing encoding");
        encoder.finish().unwrap();

        log::info!("Rendering complete");

        RenderingResult {
            output_location: output_location.into(),
        }
    }

    /// Render a single frame to a SVG document.
    fn render_frame(&self, frame: Frame) -> svg::node::element::SVG {
        let mut doc = svg::Document::new()
            .set("viewBox", (0, 0, self.width, self.height))
            .set("width", self.width)
            .set("height", self.height);

        let mut objects = frame.objects;

        for animation in frame.animations {
            let animation = animation.animate(frame.time);
            objects.push(animation);
        }

        objects.sort_by_key(|(z, _)| *z);
        for (_, object) in objects {
            doc = doc.add(object);
        }

        doc
    }

    /// Render a SVG document to a pixel buffer.
    fn render_svg(
        &self,
        doc: svg::node::element::SVG,
    ) -> ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<u8>,
        ndarray::prelude::Dim<[usize; 3]>,
    > {
        let node = convert_to_resvg(doc.to_string());
        let mut pixel_map = resvg::tiny_skia::Pixmap::new(
            self.width as u32,
            self.height as u32,
        )
        .unwrap();
        resvg::render(
            &node,
            resvg::tiny_skia::Transform::from_translate(
                self.width as f32 / 2.0,
                self.height as f32 / 2.0,
            ),
            &mut pixel_map.as_mut(),
        );
        let data = pixel_map.take();
        let mut data = ndarray::Array3::from_shape_vec(
            (self.height, self.width, 4),
            data,
        )
        .unwrap();
        data.remove_index(ndarray::Axis(2), 3);
        data.as_standard_layout().to_owned()
    }
}

/// Convert a svg string to a resvg tree.
fn convert_to_resvg(doc: String) -> resvg::usvg::Tree {
    let mut fonts = resvg::usvg::fontdb::Database::new();
    fonts.load_system_fonts();
    resvg::usvg::Tree::from_str(&doc, &Default::default(), &fonts)
        .unwrap()
}

/// The result of rendering a video.
pub struct RenderingResult {
    /// The location of the rendered video.
    pub output_location: std::path::PathBuf,
}

impl RenderingResult {
    /// Opens the rendered video in the default viewer.
    pub fn show(&self) {
        log::info!("Opening rendered video");
        let _ = open::that(&self.output_location);
    }
}
