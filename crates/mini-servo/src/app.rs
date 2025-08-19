//! Impl a simple winit app to visualize the webpage

use std::rc::Rc;

use dpi::PhysicalSize;
use servo::{create_compositor_channel, DefaultEventLoopWaker, EventLoopWaker, RenderingContext, WindowRenderingContext};
use winit::{application::ApplicationHandler, raw_window_handle::{HasDisplayHandle, HasWindowHandle}, window::{Window, WindowAttributes}};

use crate::util::make_dummy_constellation_chan;

enum AppState {
    Initial {
        size: PhysicalSize<u32>
    },
    Running,
}

pub struct App {
    state: AppState
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let AppState::Initial { size} = self.state {
            let event_loop_waker = Box::new(DefaultEventLoopWaker);
            let (compositor_proxy, compositor_receiver) =
                create_compositor_channel(event_loop_waker.clone_box());
            let compositor_api = compositor_proxy.cross_process_compositor_api.clone();
            log::info!("creating profilers");
            let time_profiler_chan = ::profile::time::Profiler::create(&None, None);
            let mem_profiler_chan = ::profile::mem::Profiler::create();
            let constellation_sender = make_dummy_constellation_chan();

            let display_handle = event_loop.display_handle().unwrap();
            let mut window_attr = Window::default_attributes();
            window_attr.inner_size = Some(winit::dpi::Size::Physical(size)); // TODO: is this necessary?
            let window = event_loop.create_window(window_attr)
                .unwrap();
            let window_handle = window.window_handle().unwrap();
            let rendering_context = WindowRenderingContext::new(display_handle, window_handle, size).unwrap();
            let rendering_context = Rc::new(rendering_context) as Rc<dyn RenderingContext>;
            
            // let handle = spawn_compositor_thread(compositor_proxy, compositor_receiver, constellation_sender, time_profiler_chan, mem_profiler_chan, event_loop_waker, rendering_context_fn);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        todo!()
    }
}