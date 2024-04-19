use rayon::prelude::*;
use std::{borrow::Cow, sync::Arc};

use video_rs::Time;

pub mod animations;
pub mod objects;

#[derive(Clone, Copy)]
pub enum Color {
    Rgba(u8, u8, u8, u8),
    Named(&'static str),
}

impl Color {
    pub fn darken(&self, amount: f32) -> Self {
        match self {
            Self::Rgba(r, g, b, a) => {
                let amount = amount.clamp(0.0, 1.0);
                let r = (*r as f32 * amount) as u8;
                let g = (*g as f32 * amount) as u8;
                let b = (*b as f32 * amount) as u8;
                Self::Rgba(r, g, b, *a)
            }
            Self::Named(_) => {
                panic!("Named colors are not supported")
            }
        }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgba(r, g, b, 255)
    }

    fn to_css(&self) -> Cow<'static, str> {
        match self {
            Self::Rgba(r, g, b, a) => {
                format!("rgba({}, {}, {}, {})", r, g, b, a).into()
            }
            Self::Named(name) => (*name).into(),
        }
    }
}

#[derive(Clone)]
struct Frame {
    time: f32,
    objects: Vec<Arc<dyn objects::Object>>,
    animations: Vec<Arc<animations::AnimationContainer>>,
}

#[derive(Default)]
pub struct Timeline {
    objects: Vec<Arc<dyn objects::Object>>,
    animations: Vec<Arc<animations::AnimatedObject>>,
}

impl Timeline {
    pub fn add_object(
        &mut self,
        object: Arc<dyn objects::Object>,
    ) -> &mut Self {
        self.objects.push(object);
        self
    }

    pub fn add_animation(
        &mut self,
        animated_object: Arc<animations::AnimatedObject>,
    ) -> &mut Self {
        self.animations.push(animated_object);
        self
    }

    fn calc_frames(&self, fps: usize) -> Vec<Frame> {
        let end_time = self
            .animations
            .iter()
            .map(|animated_object| animated_object.exit.end)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let frame_count = (end_time * fps as f32).ceil() as usize;

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

            let object = animated_object.object.clone();
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

pub struct Renderer {
    width: usize,
    height: usize,
    fps: u32,
    timeline: Timeline,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            fps: 60,
            timeline: Default::default(),
        }
    }

    pub fn set_fps(&mut self, fps: u32) -> &mut Self {
        self.fps = fps;
        self
    }

    pub fn timeline(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

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
        let frames = frames
            .into_par_iter()
            .panic_fuse()
            .map(|frame| {
                let doc = self.render_frame(frame);
                self.render_svg(doc)
            })
            .collect::<Vec<_>>();

        log::info!("Encoding frames");
        for frame in frames {
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

    fn render_frame(&self, frame: Frame) -> svg::node::element::SVG {
        let mut doc = svg::Document::new()
            .set("viewBox", (0, 0, self.width, self.height))
            .set("width", self.width)
            .set("height", self.height);

        for object in frame.objects {
            let object = object.render();
            doc = doc.add(object);
        }
        for animation in frame.animations {
            let animation = animation.animate(frame.time);
            doc = doc.add(animation);
        }

        doc
    }

    fn render_svg(
        &self,
        doc: svg::node::element::SVG,
    ) -> ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<u8>,
        ndarray::prelude::Dim<[usize; 3]>,
    > {
        let node = resvg::usvg::Tree::from_str(
            &doc.to_string(),
            &Default::default(),
            &resvg::usvg::fontdb::Database::new(),
        )
        .unwrap();
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

pub struct RenderingResult {
    pub output_location: std::path::PathBuf,
}

impl RenderingResult {
    pub fn show(&self) {
        log::info!("Opening rendered video");
        let _ = open::that(&self.output_location);
    }
}
