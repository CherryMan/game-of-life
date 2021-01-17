use super::view::*;
use super::world::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // References to useful DOM elements.
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();

    let rate = get_element(&document, "rate")?;

    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Reference to the `Game` object.
    let world = Rc::new(RefCell::new(World::new()));
    let view = Rc::new(RefCell::new(View::new()));

    let stale = Rc::new(Cell::new(true));
    let mdown = Rc::new(Cell::new(false));
    let mpos = Rc::new(Cell::new((0isize, 0isize)));

    canvas.set_width(body.client_width() as u32);
    canvas.set_height(body.client_height() as u32 * 8 / 10);

    (*view)
        .borrow_mut()
        .resize(canvas.width() as usize, canvas.height() as usize);

    // Initial pattern
    {
        let mut world = world.borrow_mut();
        world.set(2, 0);
        world.set(2, 1);
        world.set(2, 2);
        world.set(1, 2);
        world.set(0, 1);

        let mut view = view.borrow_mut();
    }

    {
        let stale = stale.clone();
        let view = view.clone();
        let window = web_sys::window().unwrap();
        let canvas = canvas.clone();
        let body = body.clone();
        let f = Closure::wrap(Box::new(move || {
            stale.set(true);
            canvas.set_width(body.client_width() as u32);
            canvas.set_height(body.client_height() as u32);
            (*view)
                .borrow_mut()
                .resize(canvas.width() as usize, canvas.height() as usize);
            stale.set(true);
        }) as Box<dyn FnMut()>);
        f.forget();
    }

    {
        let stale = stale.clone();
        let view = view.clone();
        let f = Closure::wrap(Box::new(move |e: web_sys::WheelEvent| {
            stale.set(true);

            let scale = 1.1f64.powf(-e.delta_y() / 90.);
            let x = e.offset_x() as isize;
            let y = e.offset_y() as isize;

            (*view).borrow_mut().update_scale(|s| (s * scale).clamp(2., 100.), (x, y));
            e.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", f.as_ref().unchecked_ref())?;
        f.forget();
    }

    {
        let mdown = mdown.clone();
        let mpos = mpos.clone();
        let f = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            mdown.set(true);
            mpos.set((e.offset_x() as isize, e.offset_y() as isize));
            e.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", f.as_ref().unchecked_ref())?;
        f.forget();
    }

    {
        let mdown = mdown.clone();
        let f = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            mdown.set(false);
            e.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", f.as_ref().unchecked_ref())?;
        f.forget();
    }

    {
        let mdown = mdown.clone();
        let mpos = mpos.clone();
        let stale = stale.clone();
        let view = view.clone();
        let f = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            if mdown.get() {
                let (xa, ya) = mpos.get();
                let (xb, yb) = (e.offset_x() as isize, e.offset_y() as isize);
                let (dx, dy) = (xb - xa, yb - ya);

                if dx != 0 && dy != 0 {
                    stale.set(true);
                    (*view).borrow_mut().trans(dx, dy);
                }

                mpos.set((xb, yb));
            }
            e.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", f.as_ref().unchecked_ref())?;
        f.forget();
    }

    // Reset button.
    {
        let canvas = canvas.clone();
        let ctx = ctx.clone();
        let world = world.clone();
        let view = view.clone();

        let f = Closure::wrap(Box::new(move || {
            redraw(&world.borrow(), &(*view).borrow(), &canvas, &ctx);
        }) as Box<dyn FnMut()>);

        get_element(&document, "reset-button")?.set_onclick(Some(f.as_ref().unchecked_ref()));
        f.forget();
    }

    // Play button.
    {
        let world = world.clone();
        let stale = stale.clone();

        let f = Closure::wrap(Box::new(move || {
            stale.set(true);
            world.borrow_mut().step();
        }) as Box<dyn FnMut()>);

        get_element(&document, "play-button")?.set_onclick(Some(f.as_ref().unchecked_ref()));
        f.forget();
    }

    // Build main game loop.
    {
        let stale = stale.clone();
        let canvas = canvas.clone();
        let ctx = ctx.clone();
        let world = world.clone();
        let view = view.clone();

        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();

        let window = web_sys::window().unwrap();
        if let Some(perf) = window.performance() {
            let time = RefCell::new(perf.now());
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                window
                    .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap();

                let t = perf.now();
                let dt = t - *time.borrow();

                *time.borrow_mut() = t;

                if stale.get() {
                    redraw(&world.borrow(), &(*view).borrow(), &canvas, &ctx);
                }
            }) as Box<dyn FnMut()>));
        }

        let window = web_sys::window().unwrap();
        window
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }

    Ok(())
}

fn get_element(document: &Document, id: &str) -> Result<HtmlElement, Element> {
    document
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlElement>()
}

fn redraw(w: &World, v: &View, c: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    let (width, height) = (c.width() as f64, c.height() as f64);

    ctx.set_fill_style(&JsValue::from_str("white"));
    ctx.fill_rect(0., 0., c.width().into(), c.height().into());

    ctx.set_fill_style(&JsValue::from_str("black"));
    ctx.set_line_width(1.);

    for x in v.gridlines_x() {
        ctx.fill_rect(x as f64, 0., 1., height);
    }

    for y in v.gridlines_y() {
        ctx.fill_rect(0., y as f64, width, 1.);
    }

    for (x, y, w, h) in v.rects(w.into_iter()) {
        ctx.fill_rect(x as f64, y as f64, w as f64, h as f64);
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
