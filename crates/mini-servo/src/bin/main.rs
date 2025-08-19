use std::{rc::Rc, sync::Arc};

use blitz_dom::BaseDocument;
use dpi::PhysicalSize;
use euclid::Size2D;
use layout::{context::LayoutContext, LayoutFontMetricsProvider};
use parking_lot::Mutex;
use selectors::Element;
use servo::{
    DefaultEventLoopWaker, EventLoopWaker, RenderingContext,
    SoftwareRenderingContext, create_compositor_channel
};
use servo_config::opts::DebugOptions;
use style::{
    dom::{TDocument, TNode},
    selector_parser::SnapshotMap,
    shared_lock::{SharedRwLock, StylesheetGuards},
    thread_state::{self, ThreadState},
};

use mini_servo::{
    dummy::DummyRegisteredSpeculativePainters,
    layout::layout_and_build_display_list,
    parse::ParseHtml,
    style::{resolve_style, RecalcStyle},
    util::{
        make_device, make_dummy_constellation_chan, make_font_context, make_image_resolver, make_shared_style_context, make_stylist, spawn_compositor_thread, spin_compositor_thread, RenderingContextFactory
    },
};

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

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
const DEFAULT_SIZE: PhysicalSize<u32> = PhysicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);

fn main() {
    // env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
    //     .try_init()
    //     .unwrap();
    let debug_options = DebugOptions {
        ..Default::default()
    };

    let event_loop_waker = Box::new(DefaultEventLoopWaker);
    let (compositor_proxy, compositor_receiver) =
        create_compositor_channel(event_loop_waker.clone_box());
    let compositor_api = compositor_proxy.cross_process_compositor_api.clone();
    log::info!("creating profilers");
    let time_profiler_chan = ::profile::time::Profiler::create(&None, None);
    let mem_profiler_chan = ::profile::mem::Profiler::create();
    let constellation_sender = make_dummy_constellation_chan();
    // let rendering_context = Rc::new(SoftwareRenderingContext::new(DEFAULT_SIZE).unwrap()) as Rc<dyn RenderingContext>;
    let rendering_context_factory = RenderingContextFactory::Software { size: DEFAULT_SIZE };

    let compositor_thread = spawn_compositor_thread(
        compositor_proxy,
        compositor_receiver,
        constellation_sender,
        time_profiler_chan,
        mem_profiler_chan.clone(),
        event_loop_waker,
        rendering_context_factory,
    );

    let (font_context, _storage_sender) =
        make_font_context(compositor_api.clone(), mem_profiler_chan.clone());
    let font_context = Arc::new(font_context);
    let image_resolver = Arc::new(make_image_resolver(compositor_api));

    let viewport_size = Size2D::new(DEFAULT_WIDTH as f32, DEFAULT_HEIGHT as f32);
    let device = make_device(viewport_size, Box::new(LayoutFontMetricsProvider(font_context.clone())));
    let stylist = make_stylist(device);
    let guard = SharedRwLock::new();
    let guards = StylesheetGuards {
        author: &guard.read(),
        ua_or_user: &guard.read(),
    };
    let snapshot_map = SnapshotMap::new();
    let registered_speculative_painters = DummyRegisteredSpeculativePainters;

    log::info!("creating shared style context");
    let shared_context = make_shared_style_context(
        &stylist,
        guards,
        &snapshot_map,
        &registered_speculative_painters,
    );
    let layout_context = LayoutContext {
        use_rayon: false,
        style_context: shared_context,
        font_context,
        iframe_sizes: Mutex::default(),
        image_resolver: image_resolver.clone(),
    };

    log::info!("parse document");
    thread_state::enter(ThreadState::LAYOUT);
    let doc = BaseDocument::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();

    let traversal = RecalcStyle::new(&layout_context.style_context);

    let root = TDocument::as_node(&doc.get_node(0).unwrap())
        .first_element_child()
        .unwrap()
        .as_element()
        .unwrap();

    let dirty_root = resolve_style(root, traversal, &layout_context.style_context, None).unwrap();
    assert!(dirty_root.is_html_document());

    thread_state::exit(ThreadState::LAYOUT);

    let output = layout_and_build_display_list(
        dirty_root,
        root,
        layout_context,
        &stylist,
        image_resolver,
        debug_options,
    );

    println!("Completed");
}