use dpi::PhysicalSize;
use mini_servo::app::App;
use servo_config::opts::DebugOptions;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let size = PhysicalSize::new(800, 600);
    let debug_options = DebugOptions {
        dump_display_list: true,
        dump_stacking_context_tree: true,
        dump_flow_tree: true,
        ..Default::default()
    };
    let mut app = App::new(size, debug_options, &event_loop);

    event_loop.run_app(&mut app).unwrap()
}
