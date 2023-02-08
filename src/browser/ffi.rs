use std::ffi::{CStr, CString};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::{env, io};

use libc::{c_char, c_int, c_uchar, c_uint, c_void, size_t};

use crate::gfx::{Cast, Color, Point, Rect, Size};
use crate::output::Renderer;
use crate::ui::navigation::NavigationAction;
use crate::{input, output, utils::log};

#[repr(C)]
pub struct CSize {
    width: c_uint,
    height: c_uint,
}
#[repr(C)]
pub struct CPoint {
    x: c_uint,
    y: c_uint,
}
#[repr(C)]
pub struct CRect {
    origin: CPoint,
    size: CSize,
}
#[repr(C)]
pub struct CColor {
    r: u8,
    g: u8,
    b: u8,
}

impl<T: Copy> From<&CPoint> for Point<T>
where
    c_uint: Cast<T>,
{
    fn from(value: &CPoint) -> Self {
        Point::new(value.x, value.y).cast()
    }
}
impl<T: Copy> From<&CSize> for Size<T>
where
    c_uint: Cast<T>,
{
    fn from(value: &CSize) -> Self {
        Size::new(value.width, value.height).cast()
    }
}

#[repr(C)]
pub struct BrowserDelegate {
    shutdown: extern "C" fn(),
    refresh: extern "C" fn(),
    go_to: extern "C" fn(*const c_char),
    go_back: extern "C" fn(),
    go_forward: extern "C" fn(),
    scroll: extern "C" fn(c_int),
    key_press: extern "C" fn(c_char),
    mouse_up: extern "C" fn(c_uint, c_uint),
    mouse_down: extern "C" fn(c_uint, c_uint),
    mouse_move: extern "C" fn(c_uint, c_uint),
    post_task: extern "C" fn(extern "C" fn(*mut c_void), *mut c_void),
}

struct Args {
    debug: bool,
    chromium: Vec<String>,
}

fn parse_args() -> Args {
    let mut args = Args {
        debug: false,
        chromium: Vec::new(),
    };

    for arg in env::args().skip(1) {
        if arg == "--debug" {
            args.debug = true
        } else {
            args.chromium.push(arg)
        }
    }

    args
}

fn main() -> io::Result<Option<i32>> {
    const CARBONYL_INSIDE_SHELL: &str = "CARBONYL_INSIDE_SHELL";

    if env::vars().find(|(key, value)| key == CARBONYL_INSIDE_SHELL && value == "1") != None {
        return Ok(None);
    }

    let args = parse_args();
    let mut terminal = input::Terminal::setup();
    let output = Command::new(env::current_exe()?)
        .args(args.chromium)
        .arg("--disable-threaded-scrolling")
        .arg("--disable-threaded-animation")
        .env(CARBONYL_INSIDE_SHELL, "1")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()?;

    terminal.teardown();

    let code = output.status.code().unwrap_or(127);

    if code != 0 || args.debug {
        io::stderr().write_all(&output.stderr)?;
    }

    Ok(Some(code))
}

#[no_mangle]
pub extern "C" fn carbonyl_shell_main() {
    if let Some(code) = main().unwrap() {
        std::process::exit(code)
    }
}

#[no_mangle]
pub extern "C" fn carbonyl_renderer_create() -> *mut Renderer {
    let mut renderer = Box::new(Renderer::new());
    let src = output::size().unwrap();

    log::debug!("creating renderer, terminal size: {:?}", src);

    renderer.set_size(Size::new(7, 14), src);

    Box::into_raw(renderer)
}

#[no_mangle]
pub extern "C" fn carbonyl_renderer_clear_text(renderer: *mut Renderer) {
    let renderer = unsafe { &mut *renderer };

    renderer.clear_text()
}

#[no_mangle]
pub extern "C" fn carbonyl_renderer_push_nav(
    renderer: *mut Renderer,
    url: *const c_char,
    can_go_back: bool,
    can_go_forward: bool,
) {
    let (renderer, url) = unsafe { (&mut *renderer, CStr::from_ptr(url)) };

    renderer.push_nav(url.to_str().unwrap(), can_go_back, can_go_forward)
}

#[no_mangle]
pub extern "C" fn carbonyl_renderer_set_title(renderer: *mut Renderer, title: *const c_char) {
    let (renderer, title) = unsafe { (&mut *renderer, CStr::from_ptr(title)) };

    renderer.set_title(title.to_str().unwrap()).unwrap()
}

#[no_mangle]
pub extern "C" fn carbonyl_renderer_draw_text(
    renderer: *mut Renderer,
    text: *const c_char,
    rect: *const CRect,
    color: *const CColor,
) {
    let (renderer, text, rect, color) =
        unsafe { (&mut *renderer, CStr::from_ptr(text), &*rect, &*color) };

    renderer.draw_text(
        text.to_str().unwrap(),
        Point::from(&rect.origin),
        Size::from(&rect.size),
        Color::new(color.r, color.g, color.b),
    )
}

#[no_mangle]
pub extern "C" fn carbonyl_renderer_draw_background(
    renderer: *mut Renderer,
    pixels: *mut c_uchar,
    pixels_size: size_t,
    rect: *const CRect,
) {
    let (renderer, pixels, rect) = unsafe {
        (
            &mut *renderer,
            std::slice::from_raw_parts_mut(pixels, pixels_size),
            &*rect,
        )
    };

    renderer
        .draw_background(
            pixels,
            Rect {
                origin: Point::from(&rect.origin),
                size: Size::from(&rect.size),
            },
        )
        .unwrap()
}

#[no_mangle]
pub extern "C" fn carbonyl_output_get_size(size: *mut CSize) {
    let dst = unsafe { &mut *size };
    let src = output::size().unwrap().cast::<c_uint>();

    log::debug!("terminal size: {:?}", src);

    dst.width = src.width * 7;
    dst.height = src.height * 14;
}

extern "C" fn post_task_handler(callback: *mut c_void) {
    let mut closure = unsafe { Box::from_raw(callback as *mut Box<dyn FnMut() -> io::Result<()>>) };

    closure().unwrap();
}

fn post_task<F>(handle: &extern "C" fn(extern "C" fn(*mut c_void), *mut c_void), run: F)
where
    F: FnMut() -> io::Result<()>,
{
    let closure: *mut Box<dyn FnMut() -> io::Result<()>> = Box::into_raw(Box::new(Box::new(run)));

    handle(post_task_handler, closure as *mut c_void);
}

/// Function called by the C++ code to listen for input events.
///
/// This will block so the calling code should start and own a dedicated thread.
/// It will panic if there is any error.
#[no_mangle]
pub extern "C" fn carbonyl_input_listen(renderer: *mut Renderer, delegate: *mut BrowserDelegate) {
    let char_width = 7;
    let char_height = 14;
    let BrowserDelegate {
        shutdown,
        refresh,
        go_to,
        go_back,
        go_forward,
        scroll,
        key_press,
        mouse_up,
        mouse_down,
        mouse_move,
        post_task: handle,
    } = unsafe { &*delegate };
    let dispatch = |action: NavigationAction| {
        use NavigationAction::*;

        match action {
            Ignore => (),
            Forward => return true,
            GoBack() => go_back(),
            GoForward() => go_forward(),
            Refresh() => refresh(),
            GoTo(url) => {
                let c_str = CString::new(url).unwrap();

                go_to(c_str.as_ptr())
            }
        }

        false
    };

    use input::*;

    listen(|event| {
        post_task(handle, move || {
            let renderer = unsafe { &mut *renderer };

            use Event::*;

            match event.clone() {
                Exit => (),
                Scroll { delta } => scroll(delta as c_int * char_height as c_int),
                KeyPress { ref key } => {
                    if dispatch(renderer.keypress(key)?) {
                        key_press(key.char as c_char)
                    }
                }
                MouseUp { col, row } => {
                    if dispatch(renderer.mouse_up((col as _, row as _).into())?) {
                        mouse_up(
                            (col as c_uint) * char_width,
                            (row as c_uint - 1) * char_height,
                        )
                    }
                }
                MouseDown { col, row } => {
                    if dispatch(renderer.mouse_down((col as _, row as _).into())?) {
                        mouse_down(
                            (col as c_uint) * char_width,
                            (row as c_uint - 1) * char_height,
                        )
                    }
                }
                MouseMove { col, row } => {
                    if dispatch(renderer.mouse_move((col as _, row as _).into())?) {
                        mouse_move(
                            (col as c_uint) * char_width,
                            (row as c_uint - 1) * char_height,
                        )
                    }
                }
                Terminal(terminal) => match terminal {
                    TerminalEvent::Name(name) => log::debug!("terminal name: {name}"),
                    TerminalEvent::TrueColorSupported => renderer.enable_true_color(),
                },
            };

            Ok(())
        })
    })
    .unwrap();

    let (tx, rx) = mpsc::channel();

    post_task(handle, move || {
        shutdown();
        tx.send(()).unwrap();

        Ok(())
    });

    rx.recv().unwrap();
}
