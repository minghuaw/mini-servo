//! Impl a simple winit app to visualize the webpage

use std::{rc::Rc, sync::Arc, thread::JoinHandle, time::Duration};

use blitz_dom::BaseDocument;
use compositing_traits::CrossProcessCompositorApi;
use crossbeam_channel::Sender;
use dpi::PhysicalSize;
use euclid::Size2D;
use fonts::FontContext;
use layout::{
    LayoutFontMetricsProvider,
    context::{ImageResolver, LayoutContext},
};
use parking_lot::Mutex;
use selectors::Element;
use servo::{
    DefaultEventLoopWaker, EventLoopWaker, RenderingContext, WindowRenderingContext,
    create_compositor_channel, profile_traits,
};
use servo_config::opts::DebugOptions;
use style::{
    context::RegisteredSpeculativePainters,
    dom::{TDocument, TNode},
    selector_parser::SnapshotMap,
    shared_lock::{SharedRwLock, StylesheetGuards},
    stylist::Stylist,
    thread_state::{self, ThreadState},
};
use style_traits::CSSPixel;
use webrender::Transaction;
use webrender_api::Epoch;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use crate::{
    dummy::DummyRegisteredSpeculativePainters,
    layout::{BlitzLayoutNode, LayoutOutput, layout_and_build_display_list},
    parse::ParseHtml,
    style::{RecalcStyle, resolve_style},
    util::{
        CompositorSpinner, make_device, make_dummy_constellation_chan, make_font_context,
        make_image_resolver, make_shared_style_context, make_stylist,
    },
};

// TODO: replace with some other string
const SIMPLE_TEST_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>This is title</title>
</head>
<body>
    <h1>header 1</h1>
    <p>This is a simple paragraph in my HTML document</p>
    Here's some additional text outside of the paragraph tags
    <p>This is a para</p>
</body>
</html>
"#;

// // TODO: replace with some other string
// const SIMPLE_TEST_HTML: &str = r#"
// <!DOCTYPE html>
// <html>
// <head>

// </head>
// <body>
// </body>
// </html>
// "#;

enum AppState {
    Initial {
        size: PhysicalSize<u32>,
        debug_options: Arc<DebugOptions>,
        event_loop_proxy: EventLoopProxy<()>,
    },
    Running {
        window: Window,
        rendering_context: Rc<WindowRenderingContext>,
        compositor_spinner: CompositorSpinner,
        waker_thread: JoinHandle<()>,
        main_thread: JoinHandle<()>,
    },
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(
        size: PhysicalSize<u32>,
        debug_options: DebugOptions,
        event_loop: &EventLoop<()>,
    ) -> Self {
        let event_loop_proxy = event_loop.create_proxy();

        Self {
            state: AppState::Initial {
                size,
                debug_options: Arc::new(debug_options),
                event_loop_proxy,
            },
        }
    }
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let AppState::Initial {
            size,
            debug_options,
            event_loop_proxy,
        } = &self.state
        {
            let mut window_attr = Window::default_attributes();
            window_attr.inner_size = Some(winit::dpi::Size::Physical(*size));
            let window = event_loop.create_window(window_attr).unwrap();

            let event_loop_proxy_clone = event_loop_proxy.clone();
            let waker_thread = std::thread::spawn(move || {
                let _ = event_loop_proxy_clone.send_event(());
                std::thread::sleep(std::time::Duration::from_millis(1000));
            });

            let display_handle = event_loop.display_handle().unwrap();
            let window_handle = window.window_handle().unwrap();
            let rendering_context = Rc::new(
                WindowRenderingContext::new(display_handle, window_handle, window.inner_size())
                    .unwrap(),
            );

            let _ = rendering_context.make_current();

            let event_loop_waker = Box::new(DefaultEventLoopWaker);
            let (compositor_proxy, compositor_receiver) =
                create_compositor_channel(event_loop_waker.clone_box());
            let compositor_api = compositor_proxy.cross_process_compositor_api.clone();
            let time_profiler_chan = ::profile::time::Profiler::create(&None, None);
            let mem_profiler_chan = ::profile::mem::Profiler::create();
            let constellation_sender = make_dummy_constellation_chan();

            let (txn_tx, txn_rx) = crossbeam_channel::unbounded();
            let (compositor_started_tx, compositor_started_rx) = crossbeam_channel::bounded(1);

            let compositor_spinner = CompositorSpinner::new(
                compositor_proxy,
                compositor_receiver,
                constellation_sender,
                time_profiler_chan,
                mem_profiler_chan.clone(),
                event_loop_waker,
                rendering_context.clone(),
                txn_rx,
                compositor_started_tx,
            );

            let viewport_size = Size2D::new(
                window.inner_size().width as f32,
                window.inner_size().height as f32,
            );
            let debug_options_clone = debug_options.clone();
            let main_thread = std::thread::spawn(move || {
                if let Err(_) = compositor_started_rx.recv() {
                    return;
                }

                let mut main_thread = MainThread::new(
                    compositor_api,
                    mem_profiler_chan,
                    viewport_size,
                    DummyRegisteredSpeculativePainters,
                    debug_options_clone,
                    txn_tx,
                );

                loop {
                    main_thread.parse_style_layout();

                    std::thread::sleep(Duration::from_millis(500));
                }
            });

            self.state = AppState::Running {
                window,
                rendering_context,
                compositor_spinner,
                waker_thread,
                main_thread,
            };
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: ()) {
        spin_compositor_if_app_running(self)
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                if let AppState::Running {
                    compositor_spinner,
                    rendering_context,
                    ..
                } = &mut self.state
                {
                    if !compositor_spinner.compositor.render() {
                        log::error!("Unable to render")
                    }
                    rendering_context.present();
                }
            }
            _ => {}
        }

        spin_compositor_if_app_running(self)
    }
}

fn spin_compositor_if_app_running(app: &mut App) {
    if let AppState::Running {
        window,
        compositor_spinner,
        ..
    } = &mut app.state
    {
        match compositor_spinner.spin() {
            crate::util::Running::Stop => {
                // TODO
            }
            crate::util::Running::RequestRedraw => window.request_redraw(),
            crate::util::Running::Continue => {}
        }
    }
}

pub struct MainThread<P> {
    stylist: Stylist,
    snapshot_map: SnapshotMap,
    guard: SharedRwLock,
    registered_speculative_painters: P,
    font_context: Arc<FontContext>,
    image_resolver: Arc<ImageResolver>,
    debug_options: Arc<DebugOptions>,
    txn_tx: Sender<Transaction>,
}

impl<P> MainThread<P>
where
    P: RegisteredSpeculativePainters + Send + 'static,
{
    pub fn new(
        compositor_api: CrossProcessCompositorApi,
        mem_profiler_chan: profile_traits::mem::ProfilerChan,
        viewport_size: Size2D<f32, CSSPixel>,
        registered_speculative_painters: P,
        debug_options: Arc<DebugOptions>,
        txn_tx: Sender<Transaction>,
    ) -> Self {
        let (font_context, _storage_sender) =
            make_font_context(compositor_api.clone(), mem_profiler_chan);
        let font_context = Arc::new(font_context);
        let image_resolver = Arc::new(make_image_resolver(compositor_api));

        let device = make_device(
            viewport_size,
            Box::new(LayoutFontMetricsProvider(font_context.clone())),
        );
        let stylist = make_stylist(device);
        let guard = SharedRwLock::new();
        let snapshot_map = SnapshotMap::new();

        Self {
            stylist,
            snapshot_map,
            guard,
            registered_speculative_painters,
            font_context,
            image_resolver,
            debug_options,
            txn_tx,
        }
    }

    pub fn parse_style_layout(&mut self) {
        let guards = StylesheetGuards {
            author: &self.guard.read(),
            ua_or_user: &self.guard.read(),
        };
        let shared_context = make_shared_style_context(
            &self.stylist,
            guards,
            &self.snapshot_map,
            &self.registered_speculative_painters,
        );
        let layout_context = LayoutContext {
            use_rayon: false,
            style_context: shared_context,
            font_context: self.font_context.clone(),
            iframe_sizes: Mutex::default(),
            image_resolver: self.image_resolver.clone(),
        };

        thread_state::enter(ThreadState::LAYOUT);
        let doc = BaseDocument::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();

        let traversal = RecalcStyle::new(&layout_context.style_context);

        let root_node = TDocument::as_node(&doc.get_node(0).unwrap())
            .first_element_child()
            .unwrap()
            .as_element()
            .unwrap();

        let dirty_root_node =
            resolve_style(root_node, traversal, &layout_context.style_context, None).unwrap();
        assert!(dirty_root_node.is_html_document());

        let root = BlitzLayoutNode { value: root_node };
        let dirty_root = BlitzLayoutNode {
            value: dirty_root_node,
        };

        thread_state::exit(ThreadState::LAYOUT);

        let output = layout_and_build_display_list(
            dirty_root,
            root,
            layout_context,
            &self.stylist,
            self.image_resolver.clone(),
            &self.debug_options,
        );

        let LayoutOutput {
            box_tree: _,
            fragment_tree: _,
            stacking_context_tree: _,
            pipeline_id,
            display_list,
        } = output;

        let epoch = Epoch(0);
        let mut txn = Transaction::new();

        txn.set_display_list(epoch, (pipeline_id, display_list));
        txn.set_root_pipeline(pipeline_id);
        self.txn_tx.send(txn).unwrap();
    }
}
