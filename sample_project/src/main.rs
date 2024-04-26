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

    let square = Arc::new(
        objects::Polygon::new(vec![
            (0.0, 0.0),
            (SHAPE_SCALE, 0.0),
            (SHAPE_SCALE, SHAPE_SCALE),
            (0.0, SHAPE_SCALE),
        ])
        .shift(-SHAPE_SCALE / 2.0, -SHAPE_SCALE / 2.0)
        .fill(RED)
        .outline(RED.darken(0.5)),
    );
    let mut square_anim = animations::AnimatedObject {
        object: square.clone(),
        enter: animations::FadeAnimation::new(square.clone())
            .container()
            .duration(2.0),
        exit: animations::NoAnimation.container(),
    };

    let triangle = Arc::new(
        objects::Polygon::new(vec![
            (0.0, 0.0),
            (SHAPE_SCALE, 0.0),
            (SHAPE_SCALE / 2.0, SHAPE_SCALE),
        ])
        .shift(-SHAPE_SCALE / 2.0, -SHAPE_SCALE / 2.0)
        .fill(BLUE)
        .outline(BLUE.darken(0.5)),
    );
    let mut triangle_anim = animations::AnimatedObject {
        object: triangle.clone(),
        enter: animations::PolygonMorph::new(
            square.clone(),
            triangle.clone(),
        )
        .container(),
        exit: animations::FadeAnimation::new(triangle.clone())
            .container()
            .duration(2.0)
            .reverse(),
    };

    let square_text = Arc::new(
        objects::Text::new("Square")
            .size(50.0)
            .anchor("middle")
            .at(0.0, -SHAPE_SCALE / 2.0 - 50.0),
    );
    let triangle_text = Arc::new(
        objects::Text::new("Triangle")
            .size(50.0)
            .anchor("middle")
            .at(0.0, -SHAPE_SCALE / 2.0 - 50.0),
    );

    let mut square_text_anim = animations::AnimatedObject {
        object: square_text.clone(),
        enter: animations::TextWrite::new(&square_text).container(),
        exit: animations::TextType(square_text.clone())
            .container()
            .duration(square_text.wpm(140.0))
            .reverse(),
    };
    let mut triangle_text_anim = animations::AnimatedObject {
        object: triangle_text.clone(),
        enter: animations::TextType(triangle_text.clone())
            .container()
            .duration(triangle_text.wpm(140.0)),
        exit: animations::TextWrite::new(&triangle_text)
            .container()
            .reverse(),
    };

    square_anim = square_anim.lifetime(1.0);
    square_text_anim.enter =
        square_text_anim.enter.synchronize(&square_anim.enter);
    square_text_anim.exit =
        square_text_anim.exit.after(&square_anim.exit);

    triangle_text_anim.enter =
        triangle_text_anim.enter.after(&square_text_anim.exit);

    triangle_anim.enter = triangle_anim
        .enter
        .start_with(&square_text_anim.exit)
        .end_with(&triangle_text_anim.enter);
    triangle_anim = triangle_anim.lifetime(1.0);

    triangle_text_anim.exit =
        triangle_text_anim.exit.synchronize(&triangle_anim.exit);

    timeline.add_animation(square_anim);
    timeline.add_animation(triangle_anim);
    timeline.add_animation(square_text_anim);
    timeline.add_animation(triangle_text_anim);

    app.render();
}
