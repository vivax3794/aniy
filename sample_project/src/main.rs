use std::sync::Arc;

use aniy::{animations, objects, objects::Object, Color};

const RED: Color = Color::rgb(200, 0, 0);
const BLUE: Color = Color::rgb(0, 0, 200);
const GRAY: Color = Color::rgb(100, 100, 100);

fn main() {
    env_logger::init();

    let mut app = aniy::Renderer::new(1920, 1080);
    app.set_fps(60);

    let hello_world_obj = Arc::new(
        objects::Text::new("Hello World!")
            .size(200.0)
            .anchor("middle"),
    );

    let hello_world_animation =
        Arc::new(animations::TextWrite::new(&hello_world_obj));
    let hello_world_animation =
        animations::AnimationContainer::new(hello_world_animation)
            .duration(5.0);
    let mut hello_world_animation = animations::AnimatedObject {
        object: hello_world_obj.clone(),
        enter: hello_world_animation.clone(),
        exit: hello_world_animation.reverse().duration(2.0),
    };

    let made_by_obj = Arc::new(
        objects::Text::new(
            "Made using custom animation library (by Viv)!",
        )
        .size(50.0)
        .color(GRAY)
        .anchor("middle")
        .besides(&hello_world_obj, objects::Direction::Down),
    );
    let made_by_anim =
        Arc::new(animations::TextType(made_by_obj.clone()));
    let made_by_anim =
        animations::AnimationContainer::new(made_by_anim)
            .duration(1.5);
    let mut made_by_anim = animations::AnimatedObject {
        object: made_by_obj.clone(),
        enter: made_by_anim.clone(),
        exit: made_by_anim.reverse(),
    };

    made_by_anim.enter =
        made_by_anim.enter.after(&hello_world_animation.enter);
    made_by_anim = made_by_anim.lifetime(2.0);
    hello_world_animation.exit =
        hello_world_animation.exit.after(&made_by_anim.exit);

    let timeline = app.timeline();
    timeline.add_animation(hello_world_animation);
    timeline.add_animation(made_by_anim);

    app.render();
}
