//! Framework-agnostic kittest harness backed by a fake winit 0.31 backend.
//!
//! See `examples/minimal_app.rs` for a framework-free demo

use std::cell::{Cell, RefCell};
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use accesskit::TreeUpdate;
use dpi::{PhysicalInsets, PhysicalPosition, PhysicalSize, Position, Size};
use kittest::{AccessKitNode, NodeT, Queryable, debug_fmt_node};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use winit_core::application::ApplicationHandler;
use winit_core::cursor::{Cursor, CustomCursor, CustomCursorProvider, CustomCursorSource};
use winit_core::error::RequestError;
use winit_core::event::WindowEvent;
use winit_core::event_loop::{
    ActiveEventLoop, ControlFlow, DeviceEvents, EventLoopProxy, EventLoopProxyProvider,
    OwnedDisplayHandle,
};
use winit_core::icon::Icon;
use winit_core::monitor::{Fullscreen, MonitorHandle};
use winit_core::window::{
    CursorGrabMode, ImeCapabilities, ImeRequest, ImeRequestError, ResizeDirection, Theme,
    UserAttentionType, Window, WindowAttributes, WindowButtons, WindowId, WindowLevel,
};

// ──────────────────────────────────────────────────────────────────────────────
// KittestApp — framework bridge trait
// ──────────────────────────────────────────────────────────────────────────────

/// The trait the harness requires for whatever lives behind `WinitHarness`.
pub trait KittestApp {
    /// The underlying winit `ApplicationHandler`.
    type Inner: ApplicationHandler;

    /// Mutable access to the inner handler. The harness drives winit event
    /// dispatch through this.
    fn inner(&mut self) -> &mut Self::Inner;

    /// Return the latest pending [`TreeUpdate`], consuming it.
    ///
    /// Must return `Some` on the first call (so the harness can build its
    /// initial [`kittest::State`]); after that, returning `None` means "no
    /// change this frame".
    fn take_accesskit_update(&mut self) -> Option<TreeUpdate>;

    /// Per-frame hook run by [`WinitHarness`] after dispatching queued
    /// `WindowEvent`s but before calling `about_to_wait`. Default: no-op.
    ///
    /// Framework-specific bookkeeping goes here: polling a vdom, forcing a
    /// redraw (our fake backend never dispatches `RedrawRequested`), or
    /// rebuilding an accessibility tree that isn't auto-refreshed on events.
    fn on_frame(&mut self, _event_loop: &dyn ActiveEventLoop) {
        let _ = _event_loop;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Headless raw-window-handle providers
// ──────────────────────────────────────────────────────────────────────────────

/// A `HasDisplayHandle` / `HasWindowHandle` that reports no real handle.
///
/// This is safe-by-construction: we return `HandleError::Unavailable` for both
/// `display_handle()` and `window_handle()`. Framework code that tries to
/// register with OS accessibility (via `accesskit_winit::Adapter::new` /
/// `accesskit_xplat::Adapter::with_combined_handler`) will fail fast — that is
/// the desired behaviour in a test harness.
#[derive(Debug, Default)]
struct HeadlessHandle;

impl HasDisplayHandle for HeadlessHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasWindowHandle for HeadlessHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// EventLoopProxyProvider — no-op, records wake-up calls
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct FakeProxy {
    wakes: Mutex<usize>,
}

impl EventLoopProxyProvider for FakeProxy {
    fn wake_up(&self) {
        *self.wakes.lock().unwrap() += 1;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Custom cursor provider — we never consult it, but we need the type
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct FakeCursor;

impl CustomCursorProvider for FakeCursor {
    fn is_animated(&self) -> bool {
        false
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// FakeWindow
// ──────────────────────────────────────────────────────────────────────────────

/// A headless implementation of [`winit_core::window::Window`]. Most methods
/// are no-op stubs; only id / scale_factor / surface_size / request_redraw
/// / raw-handle-like methods carry real state.
///
/// `Window` is `Send + Sync`, so internal state uses atomics / Mutex rather than
/// `Cell`. Size is encoded as `(width << 32) | height` in an `AtomicU64`.
pub struct FakeWindow {
    id: WindowId,
    scale_factor: f64,
    surface_size: AtomicU64,
    outer_size: AtomicU64,
    redraw_requested: AtomicBool,
    handle: HeadlessHandle,
}

fn pack_size(size: PhysicalSize<u32>) -> u64 {
    (u64::from(size.width) << 32) | u64::from(size.height)
}
fn unpack_size(raw: u64) -> PhysicalSize<u32> {
    PhysicalSize::new((raw >> 32) as u32, raw as u32)
}

impl Debug for FakeWindow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeWindow")
            .field("id", &self.id)
            .field("scale_factor", &self.scale_factor)
            .field("surface_size", &unpack_size(self.surface_size.load(Ordering::Relaxed)))
            .finish_non_exhaustive()
    }
}

impl FakeWindow {
    fn new(id: WindowId, surface_size: PhysicalSize<u32>, scale_factor: f64) -> Self {
        Self {
            id,
            scale_factor,
            surface_size: AtomicU64::new(pack_size(surface_size)),
            outer_size: AtomicU64::new(pack_size(surface_size)),
            redraw_requested: AtomicBool::new(false),
            handle: HeadlessHandle,
        }
    }

    /// Returns `true` if `request_redraw` has been called since the last reset.
    pub fn take_redraw_requested(&self) -> bool {
        self.redraw_requested.swap(false, Ordering::AcqRel)
    }
}

impl Window for FakeWindow {
    fn id(&self) -> WindowId {
        self.id
    }

    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    fn request_redraw(&self) {
        self.redraw_requested.store(true, Ordering::Release);
    }

    fn pre_present_notify(&self) {}
    fn reset_dead_keys(&self) {}

    fn surface_position(&self) -> PhysicalPosition<i32> {
        PhysicalPosition::new(0, 0)
    }
    fn outer_position(&self) -> Result<PhysicalPosition<i32>, RequestError> {
        Ok(PhysicalPosition::new(0, 0))
    }
    fn set_outer_position(&self, _position: Position) {}

    fn surface_size(&self) -> PhysicalSize<u32> {
        unpack_size(self.surface_size.load(Ordering::Acquire))
    }
    fn request_surface_size(&self, size: Size) -> Option<PhysicalSize<u32>> {
        let physical = size.to_physical(self.scale_factor);
        self.surface_size.store(pack_size(physical), Ordering::Release);
        self.outer_size.store(pack_size(physical), Ordering::Release);
        Some(physical)
    }
    fn outer_size(&self) -> PhysicalSize<u32> {
        unpack_size(self.outer_size.load(Ordering::Acquire))
    }
    fn safe_area(&self) -> PhysicalInsets<u32> {
        PhysicalInsets::new(0, 0, 0, 0)
    }

    fn set_min_surface_size(&self, _: Option<Size>) {}
    fn set_max_surface_size(&self, _: Option<Size>) {}
    fn surface_resize_increments(&self) -> Option<PhysicalSize<u32>> {
        None
    }
    fn set_surface_resize_increments(&self, _: Option<Size>) {}

    fn set_title(&self, _: &str) {}
    fn title(&self) -> String {
        String::new()
    }

    fn set_transparent(&self, _: bool) {}
    fn set_blur(&self, _: bool) {}
    fn set_visible(&self, _: bool) {}
    fn is_visible(&self) -> Option<bool> {
        Some(true)
    }

    fn set_resizable(&self, _: bool) {}
    fn is_resizable(&self) -> bool {
        false
    }

    fn set_enabled_buttons(&self, _: WindowButtons) {}
    fn enabled_buttons(&self) -> WindowButtons {
        WindowButtons::all()
    }

    fn set_minimized(&self, _: bool) {}
    fn is_minimized(&self) -> Option<bool> {
        Some(false)
    }
    fn set_maximized(&self, _: bool) {}
    fn is_maximized(&self) -> bool {
        false
    }

    fn set_fullscreen(&self, _: Option<Fullscreen>) {}
    fn fullscreen(&self) -> Option<Fullscreen> {
        None
    }

    fn set_decorations(&self, _: bool) {}
    fn is_decorated(&self) -> bool {
        true
    }

    fn set_window_level(&self, _: WindowLevel) {}
    fn set_window_icon(&self, _: Option<Icon>) {}

    fn request_ime_update(&self, _: ImeRequest) -> Result<(), ImeRequestError> {
        Ok(())
    }
    fn ime_capabilities(&self) -> Option<ImeCapabilities> {
        None
    }

    fn focus_window(&self) {}
    fn has_focus(&self) -> bool {
        true
    }

    fn request_user_attention(&self, _: Option<UserAttentionType>) {}

    fn set_theme(&self, _: Option<Theme>) {}
    fn theme(&self) -> Option<Theme> {
        None
    }

    fn set_content_protected(&self, _: bool) {}

    fn set_cursor(&self, _: Cursor) {}
    fn set_cursor_position(&self, _: Position) -> Result<(), RequestError> {
        Ok(())
    }
    fn set_cursor_grab(&self, _: CursorGrabMode) -> Result<(), RequestError> {
        Ok(())
    }
    fn set_cursor_visible(&self, _: bool) {}
    fn drag_window(&self) -> Result<(), RequestError> {
        Ok(())
    }
    fn drag_resize_window(&self, _: ResizeDirection) -> Result<(), RequestError> {
        Ok(())
    }
    fn show_window_menu(&self, _: Position) {}
    fn set_cursor_hittest(&self, _: bool) -> Result<(), RequestError> {
        Ok(())
    }

    fn current_monitor(&self) -> Option<MonitorHandle> {
        None
    }
    fn available_monitors(&self) -> Box<dyn Iterator<Item = MonitorHandle>> {
        Box::new(std::iter::empty())
    }
    fn primary_monitor(&self) -> Option<MonitorHandle> {
        None
    }

    fn rwh_06_display_handle(&self) -> &dyn HasDisplayHandle {
        &self.handle
    }
    fn rwh_06_window_handle(&self) -> &dyn HasWindowHandle {
        &self.handle
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// FakeActiveEventLoop
// ──────────────────────────────────────────────────────────────────────────────

/// A headless implementation of [`winit_core::event_loop::ActiveEventLoop`].
pub struct FakeActiveEventLoop {
    control_flow: Cell<ControlFlow>,
    exiting: Cell<bool>,
    windows: RefCell<Vec<Arc<FakeWindow>>>,
    next_window_id: Cell<usize>,
    proxy: Arc<FakeProxy>,
    display: Arc<HeadlessHandle>,
}

impl Debug for FakeActiveEventLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeActiveEventLoop")
            .field("exiting", &self.exiting.get())
            .field("windows", &self.windows.borrow().len())
            .finish_non_exhaustive()
    }
}

impl FakeActiveEventLoop {
    pub fn new() -> Self {
        Self {
            control_flow: Cell::new(ControlFlow::default()),
            exiting: Cell::new(false),
            windows: RefCell::new(Vec::new()),
            next_window_id: Cell::new(1),
            proxy: Arc::new(FakeProxy::default()),
            display: Arc::new(HeadlessHandle),
        }
    }

    /// Returns a clone of the first window created via `create_window`, if any.
    pub fn primary_window(&self) -> Option<Arc<FakeWindow>> {
        self.windows.borrow().first().cloned()
    }
}

impl Default for FakeActiveEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveEventLoop for FakeActiveEventLoop {
    fn create_proxy(&self) -> EventLoopProxy {
        EventLoopProxy::new(self.proxy.clone())
    }

    fn create_window(
        &self,
        attrs: WindowAttributes,
    ) -> Result<Box<dyn Window>, RequestError> {
        let id = {
            let next = self.next_window_id.get();
            self.next_window_id.set(next + 1);
            WindowId::from_raw(next)
        };
        let scale_factor = 1.0;
        let surface_size = attrs
            .surface_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(PhysicalSize::new(800, 600));
        let window = Arc::new(FakeWindow::new(id, surface_size, scale_factor));
        self.windows.borrow_mut().push(window.clone());

        // We need to return a Box<dyn Window>. `Arc<FakeWindow>` doesn't implement
        // Window directly. Build a thin newtype that derefs via the Arc.
        Ok(Box::new(SharedFakeWindow(window)))
    }

    fn create_custom_cursor(
        &self,
        _source: CustomCursorSource,
    ) -> Result<CustomCursor, RequestError> {
        Ok(CustomCursor(Arc::new(FakeCursor)))
    }

    fn available_monitors(&self) -> Box<dyn Iterator<Item = MonitorHandle>> {
        Box::new(std::iter::empty())
    }
    fn primary_monitor(&self) -> Option<MonitorHandle> {
        None
    }
    fn listen_device_events(&self, _: DeviceEvents) {}
    fn system_theme(&self) -> Option<Theme> {
        None
    }

    fn set_control_flow(&self, cf: ControlFlow) {
        self.control_flow.set(cf);
    }
    fn control_flow(&self) -> ControlFlow {
        self.control_flow.get()
    }
    fn exit(&self) {
        self.exiting.set(true);
    }
    fn exiting(&self) -> bool {
        self.exiting.get()
    }

    fn owned_display_handle(&self) -> OwnedDisplayHandle {
        OwnedDisplayHandle::new(self.display.clone())
    }
    fn rwh_06_handle(&self) -> &dyn HasDisplayHandle {
        &*self.display
    }
}

// A wrapper so `create_window` can hand out a `Box<dyn Window>` that shares
// state with the event loop (via Arc). Implements Window by delegating.
struct SharedFakeWindow(Arc<FakeWindow>);

impl Debug for SharedFakeWindow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&*self.0, f)
    }
}

impl Window for SharedFakeWindow {
    fn id(&self) -> WindowId {
        self.0.id()
    }
    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }
    fn request_redraw(&self) {
        self.0.request_redraw();
    }
    fn pre_present_notify(&self) {
        self.0.pre_present_notify();
    }
    fn reset_dead_keys(&self) {
        self.0.reset_dead_keys();
    }
    fn surface_position(&self) -> PhysicalPosition<i32> {
        self.0.surface_position()
    }
    fn outer_position(&self) -> Result<PhysicalPosition<i32>, RequestError> {
        self.0.outer_position()
    }
    fn set_outer_position(&self, p: Position) {
        self.0.set_outer_position(p);
    }
    fn surface_size(&self) -> PhysicalSize<u32> {
        self.0.surface_size()
    }
    fn request_surface_size(&self, s: Size) -> Option<PhysicalSize<u32>> {
        self.0.request_surface_size(s)
    }
    fn outer_size(&self) -> PhysicalSize<u32> {
        self.0.outer_size()
    }
    fn safe_area(&self) -> PhysicalInsets<u32> {
        self.0.safe_area()
    }
    fn set_min_surface_size(&self, s: Option<Size>) {
        self.0.set_min_surface_size(s);
    }
    fn set_max_surface_size(&self, s: Option<Size>) {
        self.0.set_max_surface_size(s);
    }
    fn surface_resize_increments(&self) -> Option<PhysicalSize<u32>> {
        self.0.surface_resize_increments()
    }
    fn set_surface_resize_increments(&self, s: Option<Size>) {
        self.0.set_surface_resize_increments(s);
    }
    fn set_title(&self, t: &str) {
        self.0.set_title(t);
    }
    fn title(&self) -> String {
        self.0.title()
    }
    fn set_transparent(&self, v: bool) {
        self.0.set_transparent(v);
    }
    fn set_blur(&self, v: bool) {
        self.0.set_blur(v);
    }
    fn set_visible(&self, v: bool) {
        self.0.set_visible(v);
    }
    fn is_visible(&self) -> Option<bool> {
        self.0.is_visible()
    }
    fn set_resizable(&self, v: bool) {
        self.0.set_resizable(v);
    }
    fn is_resizable(&self) -> bool {
        self.0.is_resizable()
    }
    fn set_enabled_buttons(&self, b: WindowButtons) {
        self.0.set_enabled_buttons(b);
    }
    fn enabled_buttons(&self) -> WindowButtons {
        self.0.enabled_buttons()
    }
    fn set_minimized(&self, v: bool) {
        self.0.set_minimized(v);
    }
    fn is_minimized(&self) -> Option<bool> {
        self.0.is_minimized()
    }
    fn set_maximized(&self, v: bool) {
        self.0.set_maximized(v);
    }
    fn is_maximized(&self) -> bool {
        self.0.is_maximized()
    }
    fn set_fullscreen(&self, f: Option<Fullscreen>) {
        self.0.set_fullscreen(f);
    }
    fn fullscreen(&self) -> Option<Fullscreen> {
        self.0.fullscreen()
    }
    fn set_decorations(&self, v: bool) {
        self.0.set_decorations(v);
    }
    fn is_decorated(&self) -> bool {
        self.0.is_decorated()
    }
    fn set_window_level(&self, l: WindowLevel) {
        self.0.set_window_level(l);
    }
    fn set_window_icon(&self, i: Option<Icon>) {
        self.0.set_window_icon(i);
    }
    fn request_ime_update(&self, r: ImeRequest) -> Result<(), ImeRequestError> {
        self.0.request_ime_update(r)
    }
    fn ime_capabilities(&self) -> Option<ImeCapabilities> {
        self.0.ime_capabilities()
    }
    fn focus_window(&self) {
        self.0.focus_window();
    }
    fn has_focus(&self) -> bool {
        self.0.has_focus()
    }
    fn request_user_attention(&self, a: Option<UserAttentionType>) {
        self.0.request_user_attention(a);
    }
    fn set_theme(&self, t: Option<Theme>) {
        self.0.set_theme(t);
    }
    fn theme(&self) -> Option<Theme> {
        self.0.theme()
    }
    fn set_content_protected(&self, v: bool) {
        self.0.set_content_protected(v);
    }
    fn set_cursor(&self, c: Cursor) {
        self.0.set_cursor(c);
    }
    fn set_cursor_position(&self, p: Position) -> Result<(), RequestError> {
        self.0.set_cursor_position(p)
    }
    fn set_cursor_grab(&self, m: CursorGrabMode) -> Result<(), RequestError> {
        self.0.set_cursor_grab(m)
    }
    fn set_cursor_visible(&self, v: bool) {
        self.0.set_cursor_visible(v);
    }
    fn drag_window(&self) -> Result<(), RequestError> {
        self.0.drag_window()
    }
    fn drag_resize_window(&self, d: ResizeDirection) -> Result<(), RequestError> {
        self.0.drag_resize_window(d)
    }
    fn show_window_menu(&self, p: Position) {
        self.0.show_window_menu(p);
    }
    fn set_cursor_hittest(&self, v: bool) -> Result<(), RequestError> {
        self.0.set_cursor_hittest(v)
    }
    fn current_monitor(&self) -> Option<MonitorHandle> {
        self.0.current_monitor()
    }
    fn available_monitors(&self) -> Box<dyn Iterator<Item = MonitorHandle>> {
        self.0.available_monitors()
    }
    fn primary_monitor(&self) -> Option<MonitorHandle> {
        self.0.primary_monitor()
    }
    fn rwh_06_display_handle(&self) -> &dyn HasDisplayHandle {
        self.0.rwh_06_display_handle()
    }
    fn rwh_06_window_handle(&self) -> &dyn HasWindowHandle {
        self.0.rwh_06_window_handle()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// WinitHarness
// ──────────────────────────────────────────────────────────────────────────────

/// A kittest harness that drives an [`ApplicationHandler`] synthetically.
pub struct WinitHarness<A: KittestApp> {
    app: A,
    event_loop: FakeActiveEventLoop,
    primary_window_id: WindowId,
    state: kittest::State,
    queued_events: Mutex<Vec<WindowEvent>>,
}

impl<A: KittestApp> WinitHarness<A> {
    /// Build the harness, mirroring winit's `EventLoop::run_app` init sequence
    /// (can_create_surfaces → resumed → on_frame → about_to_wait), then
    /// pulling the first [`TreeUpdate`] and building [`kittest::State`].
    pub fn new(build: impl FnOnce(&dyn ActiveEventLoop) -> A) -> Self {
        let event_loop = FakeActiveEventLoop::new();
        let mut app = build(&event_loop);

        app.inner().can_create_surfaces(&event_loop);
        let primary_window_id = event_loop
            .primary_window()
            .expect("app did not create a window during can_create_surfaces")
            .id();
        app.inner().resumed(&event_loop);
        app.on_frame(&event_loop);
        app.inner().about_to_wait(&event_loop);

        let initial = app
            .take_accesskit_update()
            .expect("app did not emit an initial AccessKit TreeUpdate");
        let state = kittest::State::new(initial);

        Self {
            app,
            event_loop,
            primary_window_id,
            state,
            queued_events: Mutex::new(Vec::new()),
        }
    }

    /// Enqueue a [`WindowEvent`] to be dispatched on the next [`Self::run_frame`].
    pub fn push_event(&self, event: WindowEvent) {
        self.queued_events.lock().unwrap().push(event);
    }

    /// Drain queued events through `inner.window_event`, run `on_frame`, then
    /// `inner.about_to_wait`, then pull the latest TreeUpdate into
    /// [`kittest::State`].
    pub fn run_frame(&mut self) {
        let events: Vec<WindowEvent> = {
            let mut queue = self.queued_events.lock().unwrap();
            std::mem::take(&mut *queue)
        };
        for ev in events {
            self.app
                .inner()
                .window_event(&self.event_loop, self.primary_window_id, ev);
        }
        self.app.on_frame(&self.event_loop);
        self.app.inner().about_to_wait(&self.event_loop);

        if let Some(update) = self.app.take_accesskit_update() {
            self.state.update(update);
        }
    }

    pub fn state(&self) -> &kittest::State {
        &self.state
    }
    pub fn app(&self) -> &A {
        &self.app
    }
    pub fn app_mut(&mut self) -> &mut A {
        &mut self.app
    }
    pub fn primary_window_id(&self) -> WindowId {
        self.primary_window_id
    }
}

impl<'tree, 'node, A: KittestApp> Queryable<'tree, 'node, WinitNode<'tree>> for WinitHarness<A>
where
    'node: 'tree,
{
    fn queryable_node(&'node self) -> WinitNode<'tree> {
        WinitNode {
            node: self.state.root(),
            queue: &self.queued_events,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// WinitNode
// ──────────────────────────────────────────────────────────────────────────────

/// A kittest node that can inject synthetic [`WindowEvent`]s back into the harness.
#[derive(Clone, Copy)]
pub struct WinitNode<'tree> {
    node: AccessKitNode<'tree>,
    queue: &'tree Mutex<Vec<WindowEvent>>,
}

impl<'tree> Debug for WinitNode<'tree> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        debug_fmt_node(self, f)
    }
}

impl<'tree> NodeT<'tree> for WinitNode<'tree> {
    fn accesskit_node(&self) -> AccessKitNode<'tree> {
        self.node
    }

    fn new_related(&self, child_node: AccessKitNode<'tree>) -> Self {
        Self {
            node: child_node,
            queue: self.queue,
        }
    }
}

impl<'tree> WinitNode<'tree> {
    /// Synthesize a left-mouse click at the centre of this node's AccessKit bounds.
    /// Emits PointerMoved + PointerButton pressed + PointerButton released.
    pub fn click(&self) {
        use winit_core::event::{ButtonSource, ElementState, MouseButton, PointerSource};

        let (x, y) = self.node.bounding_box().map_or((0.0, 0.0), |r| {
            ((r.x0 + r.x1) * 0.5, (r.y0 + r.y1) * 0.5)
        });
        let position = dpi::PhysicalPosition::new(x, y);
        let mut queue = self.queue.lock().unwrap();
        queue.push(WindowEvent::PointerMoved {
            device_id: None,
            position,
            primary: true,
            source: PointerSource::Mouse,
        });
        queue.push(WindowEvent::PointerButton {
            device_id: None,
            state: ElementState::Pressed,
            position,
            primary: true,
            button: ButtonSource::Mouse(MouseButton::Left),
        });
        queue.push(WindowEvent::PointerButton {
            device_id: None,
            state: ElementState::Released,
            position,
            primary: true,
            button: ButtonSource::Mouse(MouseButton::Left),
        });
    }

    /// Enqueue an AccessKit action request targeting this node. This is what
    /// `integration_example`'s egui `click()` does, and is the simplest way to
    /// trigger framework behaviour without running through hit-testing.
    pub fn accesskit_action(&self, action: accesskit::Action) {
        // Most frameworks receive AccessKit actions via a side channel (they
        // register an ActionHandler with accesskit_winit / accesskit_xplat).
        // Having the harness forward them through a WindowEvent is not
        // universally supported — so we expose this as a utility the framework
        // bridge can use, rather than a default. For the prototype we don't
        // dispatch it directly; the caller (test app) can observe this action
        // by implementing a custom dispatch on top of `take_accesskit_update`.
        let _ = action;
    }

    /// Raw access to the backing event queue, for framework-specific helpers.
    pub fn event_queue(&self) -> &'tree Mutex<Vec<WindowEvent>> {
        self.queue
    }
}

// Re-export the crates a downstream test-harness most commonly needs, so it
// doesn't have to add direct dependencies on matching versions.
pub use accesskit;
pub use kittest;
pub use winit_core;

// ──────────────────────────────────────────────────────────────────────────────
// CapturingWindowRenderer — bridge ImageRenderer → WindowRenderer
// ──────────────────────────────────────────────────────────────────────────────

/// Adapter that lets any [`anyrender::ImageRenderer`] impl satisfy
/// [`anyrender::WindowRenderer`] in a headless test harness. Each frame's
/// pixel output is stashed in a shared buffer that the test can read through
/// a [`FrameCapture`] handle.
///
/// This exists because Blitz (and other anyrender-based frameworks) ask their
/// `View` for a `WindowRenderer` — which normally needs a real OS surface —
/// but tests don't have a window. An `ImageRenderer` like
/// `anyrender_vello_cpu::VelloCpuImageRenderer` renders to a plain `Vec<u8>`
/// and doesn't need one, so this wrapper plugs the gap.
#[cfg(feature = "render")]
pub struct CapturingWindowRenderer<R: anyrender::ImageRenderer> {
    inner: Option<R>,
    width: u32,
    height: u32,
    buffer: Arc<Mutex<Frame>>,
    is_active: bool,
}

#[cfg(feature = "render")]
#[derive(Default, Clone, Debug)]
struct Frame {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

/// Read-side handle for the pixels produced by a [`CapturingWindowRenderer`].
#[cfg(feature = "render")]
#[derive(Clone)]
pub struct FrameCapture {
    buffer: Arc<Mutex<Frame>>,
}

#[cfg(feature = "render")]
impl FrameCapture {
    /// Get a snapshot of the most recently rendered frame as RGBA8 pixels,
    /// together with its width and height. Returns `None` if no frame has
    /// been rendered yet.
    pub fn latest(&self) -> Option<(Vec<u8>, u32, u32)> {
        let frame = self.buffer.lock().unwrap();
        if frame.pixels.is_empty() {
            None
        } else {
            Some((frame.pixels.clone(), frame.width, frame.height))
        }
    }

    /// Get the raw width/height of the latest frame, or `(0, 0)` if none yet.
    pub fn size(&self) -> (u32, u32) {
        let frame = self.buffer.lock().unwrap();
        (frame.width, frame.height)
    }
}

#[cfg(feature = "render")]
impl<R: anyrender::ImageRenderer + 'static> CapturingWindowRenderer<R> {
    /// Build the renderer together with its read-side handle.
    pub fn new() -> (Self, FrameCapture) {
        let buffer = Arc::new(Mutex::new(Frame::default()));
        let capture = FrameCapture {
            buffer: buffer.clone(),
        };
        (
            Self {
                inner: None,
                width: 0,
                height: 0,
                buffer,
                is_active: false,
            },
            capture,
        )
    }
}

#[cfg(feature = "render")]
impl<R: anyrender::ImageRenderer + 'static> anyrender::WindowRenderer
    for CapturingWindowRenderer<R>
{
    type ScenePainter<'a>
        = R::ScenePainter<'a>
    where
        Self: 'a;

    fn resume(&mut self, _window: Arc<dyn anyrender::WindowHandle>, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.inner = Some(R::new(width, height));
        self.is_active = true;
    }

    fn suspend(&mut self) {
        self.is_active = false;
        self.inner = None;
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let Some(inner) = &mut self.inner {
            inner.resize(width, height);
        }
    }

    fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(&mut self, draw_fn: F) {
        let Some(inner) = &mut self.inner else {
            return;
        };
        // Clear the scene between frames. `anyrender_vello` does this
        // internally in its own `render()`; `anyrender_vello_cpu` (as of 0.10)
        // does not, so scene commands accumulate and we see ghosting. We do
        // it here unconditionally — a fresh `ImageRenderer` scene is what
        // the caller expects on every `WindowRenderer::render` invocation.
        inner.reset();
        let mut frame = self.buffer.lock().unwrap();
        frame.width = self.width;
        frame.height = self.height;
        inner.render_to_vec(draw_fn, &mut frame.pixels);
    }
}
