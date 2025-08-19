use std::{cell::Cell, rc::Rc, sync::Arc};

    use base::{id::PipelineNamespace, Epoch};
    use blitz_dom::BaseDocument;
    use compositing::{IOCompositor, InitialCompositorState};
    use compositing_traits::CrossProcessCompositorApi;
    use dpi::PhysicalSize;
    use env_logger::Env;
    use euclid::Size2D;
    use layout::{
        context::{ImageResolver, LayoutContext}, display_list::StackingContextTree, BoxTree
    };
    use layout_api::LayoutDamage;
    use net::image_cache::ImageCacheImpl;
    use net_traits::image_cache::ImageCache;
    use parking_lot::{Mutex, lock_api::RwLock};
    use selectors::Element;
    use servo::{
        create_compositor_channel, profile, DefaultEventLoopWaker, EventLoopWaker, RenderingContext, ShutdownState, SoftwareRenderingContext
    };
    use servo_config::opts::DebugOptions;
    use servo_url::ImmutableOrigin;
    use style::{
        dom::{TDocument, TNode},
        selector_parser::{RestyleDamage, SnapshotMap},
        shared_lock::{SharedRwLock, StylesheetGuards},
        thread_state::{self, ThreadState},
    };
    use webrender_api::units::LayoutSize;
    use winit::raw_window_handle::DisplayHandle;

    use mini_servo::{
        dummy::DummyRegisteredSpeculativePainters,
        layout::BlitzLayoutNode,
        parse::ParseHtml,
        style::{RecalcStyle, resolve_style},
        util::{
            make_device, make_dummy_constellation_chan, make_font_context,
            make_shared_style_context, make_stylist, make_webrender, spin_compositor,
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

fn main() {
    // env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
    //     .try_init()
    //     .unwrap();

    let width = 800u32;
    let height = 600u32;

    // let compositor_api = CrossProcessCompositorApi::dummy();
    let event_loop_waker = Box::new(DefaultEventLoopWaker);
    let (compositor_proxy, compositor_receiver) =
        create_compositor_channel(event_loop_waker.clone_box());
    let compositor_api = compositor_proxy.cross_process_compositor_api.clone();
    // let mem_profiler_sender = make_dummy_mem_prof_chan();
    // let time_profiler_sender = make_dummy_time_prof_chan();
    log::info!("creating profilers");
    let time_profiler_chan = ::profile::time::Profiler::create(&None, None);
    let mem_profiler_chan = ::profile::mem::Profiler::create();
    let constellation_sender = make_dummy_constellation_chan();
    let mem_profiler_chan_clone = mem_profiler_chan.clone();

    log::info!("spawning new thread");
    let handle = std::thread::spawn(move || {
        log::info!("spawned new thread");

        let shutdown_state = Rc::new(Cell::new(ShutdownState::NotShuttingDown));
        let size = PhysicalSize::new(width, height);
        let rendering_context =
            Rc::new(SoftwareRenderingContext::new(size).unwrap()) as Rc<dyn RenderingContext>;
        let webrender_gl = rendering_context.gleam_gl_api();
        let (webrender, webrender_api_sender) = make_webrender(
            rendering_context.clone(),
            webrender_gl.clone(),
            &compositor_proxy,
        );
        let webrender_api = webrender_api_sender.create_api();
        let webrender_document =
            webrender_api.add_document(rendering_context.size2d().to_i32());

        let state = InitialCompositorState {
            sender: compositor_proxy,
            receiver: compositor_receiver,
            constellation_chan: constellation_sender,
            time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan_clone,
            shutdown_state,
            webrender,
            webrender_document,
            webrender_api,
            rendering_context,
            webrender_gl,
            event_loop_waker,
        };
        let convert_mouse_to_touch = false;

        log::info!("creating compositor");
        let mut compositor = IOCompositor::new(state, convert_mouse_to_touch);

        log::info!("created compositor");

        spin_compositor(&mut compositor);
    });

    log::info!("making font context");
    let (font_context, _storage_sender) = make_font_context(
        compositor_api.clone(),
        mem_profiler_chan.clone(),
    );
    let font_context = Arc::new(font_context);
    log::info!("creating ImageCacheImpl");
    let image_cache = ImageCacheImpl::new(compositor_api, vec![]);
    let image_resolver = Arc::new(ImageResolver {
        origin: ImmutableOrigin::new_opaque(),
        image_cache: Arc::new(image_cache),
        pending_images: Mutex::default(),
        pending_rasterization_images: Mutex::default(),
        node_to_animating_image_map: Arc::new(RwLock::default()),
        resolved_images_cache: Arc::new(RwLock::default()),
        animation_timeline_value: 0.0, // TODO: testing with 0
    });

    
    log::info!("parse document");
    thread_state::enter(ThreadState::LAYOUT);
    let doc = BaseDocument::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();

    let viewport_size = Size2D::new(width as f32, width as f32);
    let device = make_device(viewport_size);
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
    let traversal = RecalcStyle::new(&shared_context);

    let root = TDocument::as_node(&doc.get_node(0).unwrap())
        .first_element_child()
        .unwrap()
        .as_element()
        .unwrap();

    let dirty_root = resolve_style(root, traversal, &shared_context, None).unwrap();
    assert!(dirty_root.is_html_document());

    thread_state::exit(ThreadState::LAYOUT);

    let mut box_tree: Option<Arc<BoxTree>> = None;
    let restyle_damage = RestyleDamage::RELAYOUT; // TODO: 
    let layout_damage: LayoutDamage = restyle_damage.into();

    let layout_context = LayoutContext {
        use_rayon: false,
        style_context: shared_context,
        font_context,
        iframe_sizes: Mutex::default(),
        image_resolver: image_resolver.clone(),
    };

    let blitz_node = BlitzLayoutNode { value: dirty_root };
    let root_node = BlitzLayoutNode { value: root };
    if box_tree.is_none() || layout_damage.has_box_damage() {
        let mut build_box_tree = || {
            log::debug!("ran build_box_tree");
            if !BoxTree::update(&layout_context, blitz_node) {
                box_tree = Some(Arc::new(BoxTree::construct(&layout_context, root_node)));
            }
        };

        // TODO: run in parallel with rayon?
        build_box_tree();
    }

    assert!(box_tree.is_some());

    let au_viewport_size = stylist.device().au_viewport_size();
    let run_layout = || {
        box_tree
            .as_ref()
            .unwrap()
            .layout(&layout_context, au_viewport_size)
    };
    let fragment_tree = Rc::new(run_layout());

    fragment_tree.calculate_scrollable_overflow();

    // PipelineNamespace::auto_install();
    // let id = base::id::PipelineId::new();
    
    let id = webrender_api::PipelineId::dummy();
    let first_reflow = true;

    let debug_options = DebugOptions {
        ..Default::default()
    };

    let px_viewport_size = LayoutSize::new(viewport_size.width, viewport_size.height);

    // build stacking context tree
    let mut stacking_context_tree = StackingContextTree::new(
        &fragment_tree,
        px_viewport_size,
        id,
        first_reflow,
        &debug_options
    );

    // Build display list
    let compositor_info = &mut stacking_context_tree.compositor_info;
    compositor_info.hit_test_info.clear();

    let mut webrender_display_list_builder = webrender_api::DisplayListBuilder::new(compositor_info.pipeline_id);
    webrender_display_list_builder.begin();

    let mut builder = layout::display_list::DisplayListBuilder {
        current_scroll_node_id: compositor_info.root_reference_frame_id,
        current_reference_frame_scroll_node_id: compositor_info.root_reference_frame_id,
        current_clip_id: layout::display_list::clip::ClipId::INVALID,
        webrender_display_list_builder: &mut webrender_display_list_builder,
        compositor_info,
        inspector_highlight: None,
        paint_body_background: true,
        clip_map: Default::default(),
        image_resolver,
        device_pixel_ratio: stylist.device().device_pixel_ratio(),
    };

    builder.add_all_spatial_nodes();

    for clip in stacking_context_tree.clip_store.0.iter() {
        builder.add_clip_to_display_list(clip);
    }

    stacking_context_tree.root_stacking_context.build_canvas_background_display_list(&mut builder, &fragment_tree);
    stacking_context_tree.root_stacking_context.build_display_list(&mut builder);

    builder.paint_dom_inspector_highlight();

    let built_dipslay_list = webrender_display_list_builder.end().1;

    println!("Completed");
}