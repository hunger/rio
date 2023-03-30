pub mod sync;
use mio::unix::pipe::Sender;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::Arc;
use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub enum Msg {
    /// Data that should be written to the PTY.
    Input(Cow<'static, [u8]>),

    /// Indicates that the `EventLoop` should shut down, as Alacritty is shutting down.
    Shutdown,

    /// Instruction to resize the PTY.
    Resize(WindowSize),
}

#[derive(Copy, Clone, Debug)]
pub struct WindowSize {
    pub num_lines: u16,
    pub num_cols: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

#[derive(Clone)]
pub enum RioEvent {
    /// Grid has changed possibly requiring a mouse cursor shape change.
    MouseCursorDirty,

    /// Window title change.
    Title(String),

    /// Reset to the default window title.
    ResetTitle,

    /// Request to store a text string in the clipboard.
    // ClipboardStore(ClipboardType, String),

    /// Request to write the contents of the clipboard to the PTY.
    ///
    /// The attached function is a formatter which will corectly transform the clipboard content
    /// into the expected escape sequence format.
    // ClipboardLoad(ClipboardType, Arc<dyn Fn(&str) -> String + Sync + Send + 'static>),

    /// Request to write the RGB value of a color to the PTY.
    ///
    /// The attached function is a formatter which will corectly transform the RGB color into the
    /// expected escape sequence format.
    // ColorRequest(usize, Arc<dyn Fn(Rgb) -> String + Sync + Send + 'static>),

    /// Write some text to the PTY.
    PtyWrite(String),

    /// Request to write the text area size.
    TextAreaSizeRequest(Arc<dyn Fn(WindowSize) -> String + Sync + Send + 'static>),

    /// Cursor blinking state has changed.
    CursorBlinkingChange,

    /// New terminal content available.
    Wakeup,

    /// Terminal bell ring.
    Bell,

    /// Shutdown request.
    Exit,
}

impl Debug for RioEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            // RioEvent::ClipboardStore(ty, text) => write!(f, "ClipboardStore({ty:?}, {text})"),
            // RioEvent::ClipboardLoad(ty, _) => write!(f, "ClipboardLoad({ty:?})"),
            RioEvent::TextAreaSizeRequest(_) => write!(f, "TextAreaSizeRequest"),
            // RioEvent::ColorRequest(index, _) => write!(f, "ColorRequest({index})"),
            RioEvent::PtyWrite(text) => write!(f, "PtyWrite({text})"),
            RioEvent::Title(title) => write!(f, "Title({title})"),
            RioEvent::CursorBlinkingChange => write!(f, "CursorBlinkingChange"),
            RioEvent::MouseCursorDirty => write!(f, "MouseCursorDirty"),
            RioEvent::ResetTitle => write!(f, "ResetTitle"),
            RioEvent::Wakeup => write!(f, "Wakeup"),
            RioEvent::Bell => write!(f, "Bell"),
            RioEvent::Exit => write!(f, "Exit"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RioEventType {
    ScaleFactorChanged(f64, (u32, u32)),
    Rio(RioEvent),
    // ConfigReload(PathBuf),
    // Message(Message),
    // Scroll(Scroll),
    BlinkCursor,
    BlinkCursorTimeout,
    SearchNext,
    Frame,
}

impl From<RioEvent> for RioEventType {
    fn from(rio_event: RioEvent) -> Self {
        Self::Rio(rio_event)
    }
}

#[derive(Debug, Clone)]
pub struct EventP {
    /// Event payload.
    pub payload: RioEventType,
}

impl EventP {
    pub fn new(payload: RioEventType) -> Self {
        Self { payload }
    }
}

impl From<EventP> for winit::event::Event<'_, EventP> {
    fn from(event: EventP) -> Self {
        winit::event::Event::UserEvent(event)
    }
}

pub trait OnResize {
    fn on_resize(&mut self, window_size: WindowSize);
}

/// Event Loop for notifying the renderer about terminal events.
pub trait EventListener {
    fn send_event(&self, _event: RioEvent) {}
}

// pub struct Notifier(pub Sender<Msg>);
pub struct Notifier(pub Sender);

/// Byte sequences are sent to a `Notify` in response to some events.
pub trait Notify {
    /// Notify that an escape sequence should be written to the PTY.
    ///
    /// TODO this needs to be able to error somehow.
    fn notify<B: Into<Cow<'static, [u8]>>>(&self, _: B);
}

impl Notify for Notifier {
    fn notify<B>(&self, bytes: B)
    where
        B: Into<Cow<'static, [u8]>>,
    {
        let bytes = bytes.into();
        // terminal hangs if we send 0 bytes through.
        if bytes.len() == 0 {
            return;
        }

        // let _ = self.0.send(Msg::Input(bytes));
    }
}

impl OnResize for Notifier {
    fn on_resize(&mut self, window_size: WindowSize) {
        // let _ = self.0.send(Msg::Resize(window_size));
    }
}

pub struct VoidListener;

impl EventListener for VoidListener {}

#[derive(Debug, Clone)]
pub struct EventProxy {
    proxy: EventLoopProxy<EventP>,
}

impl EventProxy {
    pub fn new(proxy: EventLoopProxy<EventP>) -> Self {
        Self { proxy }
    }

    /// Send an event to the event loop.
    pub fn send_event(&self, event: RioEventType) {
        let _ = self.proxy.send_event(EventP::new(event));
    }
}

impl EventListener for EventProxy {
    fn send_event(&self, event: RioEvent) {
        let _ = self.proxy.send_event(EventP::new(event.into()));
    }
}
