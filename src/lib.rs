use js_sys::Math::random;
use std::cell::Cell;
use std::cell::RefCell;
use std::fmt::format;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::Document;
use web_sys::HtmlCanvasElement;

struct RenderLoop {
    animation_id: Option<i32>,
    pub closure: Option<Closure<dyn Fn()>>,
}

impl RenderLoop {
    pub fn new() -> RenderLoop {
        RenderLoop {
            animation_id: Some(0),
            closure: None,
        }
    }
}

fn get_window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn get_document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}
fn get_canvas(document: &Document) -> HtmlCanvasElement {
    return document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
}

fn create_canvas(document: &Document) -> HtmlCanvasElement {
    return document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = get_document();
    let canvas = create_canvas(&document);

    document.body().unwrap().append_child(&canvas)?;
    canvas.set_width(640);
    canvas.set_height(480);
    canvas.set_id("canvas");
    canvas.style().set_property("border", "solid")?;

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let context = Rc::new(context);
    let pressed = Rc::new(Cell::new(false));
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            context.begin_path();
            context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            pressed.set(true);
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            if pressed.get() {
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();
                context.begin_path();
                context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            }
        });
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            pressed.set(false);
            context.line_to(event.offset_x() as f64, event.offset_y() as f64);
            context.stroke_rect(event.offset_x() as f64, event.offset_y() as f64, 10.0, 10.0);
            context.stroke();
        });
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let render_loop: Rc<RefCell<RenderLoop>> = Rc::new(RefCell::new(RenderLoop::new()));
    {
        let closure: Closure<dyn Fn()> = {
            let render_loop = render_loop.clone();
            Closure::wrap(Box::new(move || {
                let mut render_loop = render_loop.borrow_mut();
                render_loop.animation_id = if let Some(ref closure) = render_loop.closure {
                    let document = get_document();
                    let canvas = get_canvas(&document);

                    canvas.set_width(640);
                    canvas.set_height(480);
                    let context = canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<web_sys::CanvasRenderingContext2d>()
                        .unwrap();
                    context.fill_rect(random() * 100.0, random() * 100.0, 50.0, 50.0);
                    Some(
                        get_window()
                            .request_animation_frame(closure.as_ref().unchecked_ref())
                            .expect("cannot set animation frame"),
                    )
                } else {
                    None
                }
            }))
        };
        let mut render_loop = render_loop.borrow_mut();
        render_loop.animation_id = Some(
            get_window()
                .request_animation_frame(closure.as_ref().unchecked_ref())
                .expect("cannot set animation frame"),
        );
        render_loop.closure = Some(closure);
    }
    Ok(())
}
