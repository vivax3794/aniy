use std::sync::Arc;

use aniy::{animations, objects, Color};

const COLOR: Color = Color::rgb(255, 0, 0);

fn main() {
    env_logger::init();

    let mut app = aniy::Renderer::new(1920, 1080);
    app.set_fps(30);

    let timeline = app.timeline();
    let triangle = Arc::new(
        objects::Polygon::default()
            .add_point(-530.0, 234.0)
            .add_point(200.0, 0.0)
            .add_point(420.0, -320.0)
            .fill(COLOR)
            .outline(COLOR.darken(0.5)),
    );
    let animation =
        Arc::new(animations::PolygonAnimation(triangle.clone()));
    let enter =
        animations::AnimationContainer::new(animation).duration(3.0);
    let exit = enter.clone().reverse();
    let triangle = Arc::new(
        animations::AnimatedObject {
            object: triangle,
            enter,
            exit,
        }
        .lifetime(1.0),
    );

    timeline.add_animation(triangle);

    app.render().show();
}
