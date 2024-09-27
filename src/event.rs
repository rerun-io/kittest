use accesskit::Vec2;

/// Kittest Event.
pub enum Event {
    /// An ActionRequest event. When using an application these would be generated by the
    /// screen reader.
    ActionRequest(accesskit::ActionRequest),
    /// A Simulated event, e.g. clicks or typing.
    Simulated(SimulatedEvent),
}

/// A Simulated Event. These should map to the event type of the gui framework.
///
/// The structure is inspired by the `winit` `WindowEvent` types.
pub enum SimulatedEvent {
    CursorMoved {
        position: Vec2,
    },
    MouseInput {
        state: ElementState,
        button: MouseButton,
    },
    Ime(String),
    KeyInput {
        state: ElementState,
        key: Key,
    },
}

/// The state of an element (e.g. Button), either pressed or released.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementState {
    Pressed,
    Released,
}

impl ElementState {
    /// Returns an iterator with first the Pressed state and then the Released state.
    pub fn click() -> impl Iterator<Item = Self> {
        [Self::Pressed, Self::Released].into_iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

/// The keys (currently these match egui's keys).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    // ----------------------------------------------
    // Modifier keys:
    Alt,
    Command,
    Control,
    Shift,

    // ----------------------------------------------
    // Commands:
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,

    Escape,
    Tab,
    Backspace,
    Enter,
    Space,

    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    Copy,
    Cut,
    Paste,

    // ----------------------------------------------
    // Punctuation:
    /// `:`
    Colon,

    /// `,`
    Comma,

    /// `\`
    Backslash,

    /// `/`
    Slash,

    /// `|`, a vertical bar
    Pipe,

    /// `?`
    Questionmark,

    // `[`
    OpenBracket,

    // `]`
    CloseBracket,

    /// \`, also known as "backquote" or "grave"
    Backtick,

    /// `-`
    Minus,

    /// `.`
    Period,

    /// `+`
    Plus,

    /// `=`
    Equals,

    /// `;`
    Semicolon,

    /// `'`
    Quote,

    // ----------------------------------------------
    // Digits:
    /// `0` (from main row or numpad)
    Num0,

    /// `1` (from main row or numpad)
    Num1,

    /// `2` (from main row or numpad)
    Num2,

    /// `3` (from main row or numpad)
    Num3,

    /// `4` (from main row or numpad)
    Num4,

    /// `5` (from main row or numpad)
    Num5,

    /// `6` (from main row or numpad)
    Num6,

    /// `7` (from main row or numpad)
    Num7,

    /// `8` (from main row or numpad)
    Num8,

    /// `9` (from main row or numpad)
    Num9,

    // ----------------------------------------------
    // Letters:
    A, // Used for cmd+A (select All)
    B,
    C, // |CMD COPY|
    D, // |CMD BOOKMARK|
    E, // |CMD SEARCH|
    F, // |CMD FIND firefox & chrome|
    G, // |CMD FIND chrome|
    H, // |CMD History|
    I, // italics
    J, // |CMD SEARCH firefox/DOWNLOAD chrome|
    K, // Used for ctrl+K (delete text after cursor)
    L,
    M,
    N,
    O, // |CMD OPEN|
    P, // |CMD PRINT|
    Q,
    R, // |CMD REFRESH|
    S, // |CMD SAVE|
    T, // |CMD TAB|
    U, // Used for ctrl+U (delete text before cursor)
    V, // |CMD PASTE|
    W, // Used for ctrl+W (delete previous word)
    X, // |CMD CUT|
    Y,
    Z, // |CMD UNDO|

    // ----------------------------------------------
    // Function keys:
    F1,
    F2,
    F3,
    F4,
    F5, // |CMD REFRESH|
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
    // When adding keys, remember to also update:
    // * crates/egui-winit/src/lib.rs
    // * Key::ALL
    // * Key::from_name
    // You should test that it works using the "Input Event History" window in the egui demo app.
    // Make sure to test both natively and on web!
    // Also: don't add keys last; add them to the group they best belong to.
}
