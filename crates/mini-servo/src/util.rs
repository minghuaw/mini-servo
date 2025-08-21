use std::{cell::Cell, rc::Rc, sync::Arc, thread::JoinHandle};

use compositing::{IOCompositor, InitialCompositorState};
use compositing_traits::{CompositorMsg, CompositorProxy, CrossProcessCompositorApi};
use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::{Receiver, Sender};
use dpi::PhysicalSize;
use euclid::{Scale, Size2D};
use fonts::{FontContext, SystemFontService};
use gleam::gl::Gl;
use ipc_channel::ipc::{self, IpcSender};
use layout::context::ImageResolver;
use net::{
    image_cache::ImageCacheImpl, indexeddb::IndexedDBThreadFactory,
    storage_thread::StorageThreadFactory,
};
use net_traits::{
    CoreResourceThread, ResourceThreads, image_cache::ImageCache, storage_thread::StorageThreadMsg,
};
use parking_lot::{Mutex, RwLock};
use servo::{
    EventLoopWaker, RenderNotifier, RenderingContext, ShutdownState, SoftwareRenderingContext,
    WindowRenderingContext,
    gl::RENDERER,
    profile_traits::{self, mem},
};
use servo_config::pref;
use servo_url::ImmutableOrigin;
use style::{
    animation::DocumentAnimationSet,
    context::{QuirksMode, RegisteredSpeculativePainters, SharedStyleContext},
    global_style_data::GLOBAL_STYLE_DATA,
    media_queries::{Device, MediaType},
    properties::{ComputedValues, style_structs::Font},
    queries::values::PrefersColorScheme,
    selector_parser::SnapshotMap,
    servo::media_queries::FontMetricsProvider,
    shared_lock::StylesheetGuards,
    stylist::Stylist,
    traversal_flags::TraversalFlags,
};
use style_traits::CSSPixel;
use webrender::{
    ONE_TIME_USAGE_HINT, RenderApiSender, Renderer, ShaderPrecacheFlags, Transaction, UploadMethod,
};
use webrender_api::{ColorF, RenderReasons};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub fn make_device(
    viewport_size: Size2D<f32, CSSPixel>,
    font_metrics_provider: Box<dyn FontMetricsProvider>,
) -> Device {
    let font = Font::initial_values();

    Device::new(
        MediaType::screen(),
        QuirksMode::NoQuirks,
        viewport_size,
        Scale::new(1.0),
        font_metrics_provider,
        ComputedValues::initial_values_with_font_override(font),
        PrefersColorScheme::Light,
    )
}

pub fn make_stylist(device: Device) -> Stylist {
    Stylist::new(device, QuirksMode::NoQuirks)
}

pub fn make_shared_style_context<'a>(
    stylist: &'a Stylist,
    guards: StylesheetGuards<'a>,
    snapshot_map: &'a SnapshotMap,
    registered_speculative_painters: &'a dyn RegisteredSpeculativePainters,
) -> SharedStyleContext<'a> {
    SharedStyleContext {
        stylist,
        visited_styles_enabled: false,
        options: GLOBAL_STYLE_DATA.options.clone(),
        guards,
        current_time_for_animations: 0.0,
        traversal_flags: TraversalFlags::empty(),
        snapshot_map,
        animations: DocumentAnimationSet::default(),
        registered_speculative_painters,
    }
}

pub fn make_dummy_constellation_chan() -> crossbeam_channel::Sender<EmbedderToConstellationMessage>
{
    let (sender, receiver) = crossbeam_channel::unbounded::<EmbedderToConstellationMessage>();

    // Spawn a thread that recvs messages
    // The only messsage that is sent from RefreshDriver is TickAnimation
    // So it's okay to just drop the message
    std::thread::spawn(move || {
        while let Ok(msg) = receiver.recv() {
            let s: &'static str = msg.into();
            log::debug!("dummy constellation recved msg: {:?}", s);
        }
    });

    sender
}

pub fn make_dummy_core_thread() -> CoreResourceThread {
    let (sender, recver) = ipc::channel().unwrap();

    std::thread::spawn(move || {
        while let Ok(msg) = recver.recv() {
            log::debug!("dummy core threads recved msg: {:?}", msg);
        }
    });

    sender
}

pub fn make_font_context(
    compositor_api: CrossProcessCompositorApi,
    memory_profiler_sender: mem::ProfilerChan,
) -> (FontContext, IpcSender<StorageThreadMsg>) {
    let system_font_service_proxy = Arc::new(
        SystemFontService::spawn(compositor_api.clone(), memory_profiler_sender.clone()).to_proxy(),
    );

    let core_thread = make_dummy_core_thread();
    let config_dir = None;
    let storage: IpcSender<StorageThreadMsg> =
        StorageThreadFactory::new(config_dir.clone(), memory_profiler_sender);
    let idb = IndexedDBThreadFactory::new(config_dir);
    let resource_threads = ResourceThreads::new(core_thread, storage.clone(), idb);

    let font_context =
        FontContext::new(system_font_service_proxy, compositor_api, resource_threads);
    (font_context, storage)
}

pub fn make_image_resolver(compositor_api: CrossProcessCompositorApi) -> ImageResolver {
    let image_cache = ImageCacheImpl::new(compositor_api, vec![]);
    ImageResolver {
        origin: ImmutableOrigin::new_opaque(),
        image_cache: Arc::new(image_cache),
        pending_images: Mutex::default(),
        pending_rasterization_images: Mutex::default(),
        node_to_animating_image_map: Arc::new(RwLock::default()),
        resolved_images_cache: Arc::new(RwLock::default()),
        animation_timeline_value: 0.0, // TODO: testing with 0
    }
}

pub fn make_webrender(
    rendering_context: Rc<dyn RenderingContext>,
    webrender_gl: Rc<dyn Gl>,
    compositor_proxy: &CompositorProxy,
) -> (Renderer, RenderApiSender) {
    let mut debug_flags = webrender::DebugFlags::empty();
    debug_flags.set(webrender::DebugFlags::PROFILER_DBG, false);

    rendering_context.prepare_for_rendering();
    let render_notifier = Box::new(RenderNotifier::new(compositor_proxy.clone()));
    let clear_color = servo_config::pref!(shell_background_color_rgba);
    let clear_color = ColorF::new(
        clear_color[0] as f32,
        clear_color[1] as f32,
        clear_color[2] as f32,
        clear_color[3] as f32,
    );

    // Use same texture upload method as Gecko with ANGLE:
    // https://searchfox.org/mozilla-central/source/gfx/webrender_bindings/src/bindings.rs#1215-1219
    let upload_method = if webrender_gl.get_string(RENDERER).starts_with("ANGLE") {
        UploadMethod::Immediate
    } else {
        UploadMethod::PixelBuffer(ONE_TIME_USAGE_HINT)
    };
    let worker_threads = std::thread::available_parallelism()
        .map(|i| i.get())
        .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
        .min(pref!(threadpools_webrender_workers_max).max(1) as usize);
    let workers = Some(Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(worker_threads)
            .thread_name(|idx| format!("WRWorker#{}", idx))
            .build()
            .unwrap(),
    ));
    webrender::create_webrender_instance(
        webrender_gl.clone(),
        render_notifier,
        webrender::WebRenderOptions {
            // We force the use of optimized shaders here because rendering is broken
            // on Android emulators with unoptimized shaders. This is due to a known
            // issue in the emulator's OpenGL emulation layer.
            // See: https://github.com/servo/servo/issues/31726
            use_optimized_shaders: true,
            resource_override_path: None,
            debug_flags,
            precache_flags: if pref!(gfx_precache_shaders) {
                ShaderPrecacheFlags::FULL_COMPILE
            } else {
                ShaderPrecacheFlags::empty()
            },
            enable_aa: pref!(gfx_text_antialiasing_enabled),
            enable_subpixel_aa: pref!(gfx_subpixel_text_antialiasing_enabled),
            allow_texture_swizzling: pref!(gfx_texture_swizzling_enabled),
            clear_color,
            upload_method,
            workers,
            size_of_op: Some(servo_allocator::usable_size),
            ..Default::default()
        },
        None,
    )
    .expect("Unable to initialize webrender!")
}

pub struct CompositorStarted;

pub enum Running {
    Stop,
    RequestRedraw,
    Continue,
}

pub struct CompositorSpinner {
    pub shutdown_state: Rc<Cell<ShutdownState>>,
    pub compositor: IOCompositor,
    txn_rx: Receiver<Transaction>,
}

impl CompositorSpinner {
    pub fn new(
        compositor_proxy: CompositorProxy,
        compositor_receiver: Receiver<CompositorMsg>,
        constellation_sender: Sender<EmbedderToConstellationMessage>,
        time_profiler_chan: profile_traits::time::ProfilerChan,
        mem_profiler_chan: profile_traits::mem::ProfilerChan,
        event_loop_waker: Box<dyn EventLoopWaker>,
        rendering_context: Rc<dyn RenderingContext>,
        txn_rx: Receiver<Transaction>,
        compositor_started: Sender<CompositorStarted>,
    ) -> Self {
        let shutdown_state = Rc::new(Cell::new(ShutdownState::NotShuttingDown));

        let webrender_gl = rendering_context.gleam_gl_api();
        let (webrender, webrender_api_sender) = make_webrender(
            rendering_context.clone(),
            webrender_gl.clone(),
            &compositor_proxy,
        );
        let webrender_api = webrender_api_sender.create_api();
        let webrender_document = webrender_api.add_document(rendering_context.size2d().to_i32());

        let state = InitialCompositorState {
            sender: compositor_proxy,
            receiver: compositor_receiver,
            constellation_chan: constellation_sender,
            time_profiler_chan,
            mem_profiler_chan,
            shutdown_state: shutdown_state.clone(),
            webrender,
            webrender_document,
            webrender_api,
            rendering_context,
            webrender_gl,
            event_loop_waker,
        };
        let convert_mouse_to_touch = false;

        let compositor = IOCompositor::new(state, convert_mouse_to_touch);

        let _ = compositor_started.send(CompositorStarted);

        Self {
            shutdown_state,
            compositor,
            txn_rx,
        }
    }

    pub fn spin(&mut self) -> Running {
        let mut msgs = Vec::new();
        loop {
            match self.compositor.receiver().try_recv() {
                Ok(msg) => msgs.push(msg),
                Err(err) => match err {
                    crossbeam_channel::TryRecvError::Empty => break,
                    crossbeam_channel::TryRecvError::Disconnected => return Running::Stop,
                },
            };
        }
        self.compositor.handle_messages(msgs);
        self.compositor.perform_updates();

        match self.txn_rx.try_recv() {
            Ok(mut txn) => {
                self.compositor
                    .generate_frame(&mut txn, RenderReasons::empty());
                let global = self.compositor.global();
                let mut servo_renderer = global.borrow_mut();
                servo_renderer.send_transaction(txn);
                Running::RequestRedraw
            }
            Err(err) => match err {
                crossbeam_channel::TryRecvError::Empty => Running::Continue,
                crossbeam_channel::TryRecvError::Disconnected => Running::Stop,
            },
        }
    }
}

pub fn spin_compositor_thread(
    compositor_proxy: CompositorProxy,
    compositor_receiver: Receiver<CompositorMsg>,
    constellation_sender: Sender<EmbedderToConstellationMessage>,
    time_profiler_chan: profile_traits::time::ProfilerChan,
    mem_profiler_chan: profile_traits::mem::ProfilerChan,
    event_loop_waker: Box<dyn EventLoopWaker>,
    rendering_context: Rc<dyn RenderingContext>,
    txn_rx: Receiver<Transaction>,
    compositor_started: Sender<CompositorStarted>,
) {
    let mut spinner = CompositorSpinner::new(
        compositor_proxy,
        compositor_receiver,
        constellation_sender,
        time_profiler_chan,
        mem_profiler_chan,
        event_loop_waker,
        rendering_context,
        txn_rx,
        compositor_started,
    );

    loop {
        if let Running::Stop = spinner.spin() {
            break;
        }
    }
}

pub enum RenderingContextFactory {
    Software {
        size: PhysicalSize<u32>,
    },
    Window {
        window: Window,
        size: PhysicalSize<u32>,
    },
}

impl RenderingContextFactory {
    pub fn create(self) -> Rc<dyn RenderingContext> {
        match self {
            RenderingContextFactory::Software { size } => {
                SoftwareRenderingContext::new(size).map(Rc::new).unwrap()
            }
            RenderingContextFactory::Window { window, size } => {
                let display_handle = window.display_handle().unwrap();
                let window_handle = window.window_handle().unwrap();
                WindowRenderingContext::new(display_handle, window_handle, size)
                    .map(Rc::new)
                    .unwrap()
            }
        }
    }
}

pub fn spawn_compositor_thread(
    compositor_proxy: CompositorProxy,
    compositor_receiver: Receiver<CompositorMsg>,
    constellation_sender: Sender<EmbedderToConstellationMessage>,
    time_profiler_chan: profile_traits::time::ProfilerChan,
    mem_profiler_chan: profile_traits::mem::ProfilerChan,
    event_loop_waker: Box<dyn EventLoopWaker>,
    rendering_context_factory: RenderingContextFactory,
    txn_rx: Receiver<Transaction>,
    compositor_started: Sender<CompositorStarted>,
) -> JoinHandle<()> {
    std::thread::spawn(|| {
        let rendering_context = rendering_context_factory.create();
        spin_compositor_thread(
            compositor_proxy,
            compositor_receiver,
            constellation_sender,
            time_profiler_chan,
            mem_profiler_chan,
            event_loop_waker,
            rendering_context,
            txn_rx,
            compositor_started,
        );
    })
}

#[cfg(test)]
mod tests {
    use compositing_traits::CompositorMsg;
    use servo::{DefaultEventLoopWaker, EventLoopWaker, create_compositor_channel};
    use style::shared_lock::SharedRwLock;

    use crate::dummy::{DummyFontMetricsProvider, DummyRegisteredSpeculativePainters};

    use super::*;

    #[test]
    fn test_make_shared_style_context() {
        let device = make_device(
            Size2D::new(800.0, 600.0),
            Box::new(DummyFontMetricsProvider),
        );
        let stylist = make_stylist(device);
        let guard = SharedRwLock::new();
        let guards = StylesheetGuards {
            author: &guard.read(),
            ua_or_user: &guard.read(),
        };
        let snapshot_map = SnapshotMap::new();
        let registered_speculative_painters = DummyRegisteredSpeculativePainters;

        let _shared_style_context = make_shared_style_context(
            &stylist,
            guards,
            &snapshot_map,
            &registered_speculative_painters,
        );
    }

    #[test]
    fn test_starting_compositor_channel() {
        let event_loop_waker = Box::new(DefaultEventLoopWaker) as Box<dyn EventLoopWaker>;
        let (compositor_proxy, compositor_receiver) = create_compositor_channel(event_loop_waker);

        let msg = CompositorMsg::IsReadyToSaveImageReply(false);

        compositor_proxy.send(msg);

        let recved = compositor_receiver.recv().unwrap();
        assert!(matches!(
            recved,
            CompositorMsg::IsReadyToSaveImageReply(false)
        ));
    }
}
