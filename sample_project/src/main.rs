use std::sync::Arc;

use aniy::{animations, objects, Color};

const RED: Color = Color::rgb(200, 0, 0);
const BLUE: Color = Color::rgb(0, 0, 200);

fn main() {
    env_logger::init();

    let mut app = aniy::Renderer::new(1920, 1080);
    app.set_fps(30);

    let timeline = app.timeline();

    let square_red = Arc::new(
        objects::Polygon::new([
            (-200.0, -200.0),
            (200.0, -200.0),
            (200.0, 200.0),
            (-200.0, 200.0),
        ])
        .fill(RED)
        .outline(RED.darken(0.5)),
    );
    let square_red_anim = Arc::new(
        animations::AnimatedObject {
            object: square_red.clone(),
            exit: animations::AnimationContainer::new(Arc::new(
                animations::NoAnimation,
            ))
            .duration(0.0),
            enter: animations::AnimationContainer::new(Arc::new(
                animations::PolygonDraw(Arc::clone(&square_red)),
            ))
            .duration(1.5),
        }
        .lifetime(1.0),
    );
    timeline.add_animation(square_red_anim.clone());

    let blue_square = Arc::new(
        objects::Polygon::new([
            (-300.0, -150.0),
            (400.0, -70.0),
            (230.0, 140.0),
            (-200.0, 200.0),
        ])
        .shift(100.0, -200.0)
        .fill(BLUE)
        .outline(BLUE.darken(0.5)),
    );
    let blue_square_anim = Arc::new(
        animations::AnimatedObject {
            object: blue_square.clone(),
            exit: animations::AnimationContainer::new(Arc::new(
                animations::PolygonDraw(blue_square.clone()),
            ))
            .reverse()
            .duration(1.5),
            enter: animations::AnimationContainer::new(Arc::new(
                animations::PolygonMorph(
                    square_red.clone(),
                    blue_square.clone(),
                ),
            ))
            .duration(1.5)
            .after(&square_red_anim.exit),
        }
        .lifetime(1.0),
    );
    timeline.add_animation(blue_square_anim);

    app.render().show();
}
