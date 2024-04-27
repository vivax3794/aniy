use std::sync::Arc;

use aniy::{
    animations::{self, Animation},
    objects::{self, Object},
    Color,
};

const RED: Color = Color::rgb(200, 0, 0);
const BLUE: Color = Color::rgb(0, 0, 200);
const GRAY: Color = Color::rgb(100, 100, 100);

const SHAPE_SCALE: f32 = 300.0;

fn main() {
    env_logger::init();

    let mut app = aniy::Renderer::new(1920, 1080);
    app.set_fps(60);

    let timeline = app.timeline();

    let text = Arc::new(
        objects::Math::new(
            r"
\mathbf{A} = \begin{bmatrix}
a_{11} & a_{12} & \cdots & a_{1n} \\
a_{21} & a_{22} & \cdots & a_{2n} \\
\vdots  & \vdots         & \ddots & \vdots  \\
a_{m1} & a_{m2} & \cdots & a_{mn}
\end{bmatrix}, \quad
\mathbf{B} = \begin{bmatrix}
b_{11} & b_{12} & \cdots & b_{1n} \\
b_{21} & b_{22} & \cdots & b_{2n} \\
\vdots  & \vdots         & \ddots & \vdots  \\
b_{m1} & b_{m2} & \cdots & b_{mn}
\end{bmatrix}
            ",
        )
        .size(5.0)
        .center_on(0.0, 0.0),
    );
    let text_anim = animations::AnimatedObject {
        object: text.clone(),
        enter: animations::SvgTyper::new(text.as_ref())
            .container()
            .duration(5.0),
        exit: animations::FadeGradient::new(text.as_ref())
            .container()
            .reverse()
            .duration(1.0),
    }
    .lifetime(1.0);

    timeline.add_animation(text_anim);

    app.render();
}
