use super::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

enum Event {
    Reset,
    TogglePause,
    Step,
    SetRate(f64),
    ToggleCell(u16, u16),
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // References to useful DOM elements.
    let document = web_sys::window().unwrap().document().unwrap();

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
    let game = Rc::new(RefCell::new(Game::new(64, 48)));

    // Initial pattern
    {
        let mut game = game.borrow_mut();
        game.set(2, 0);
        game.set(2, 1);
        game.set(2, 2);
        game.set(1, 2);
        game.set(0, 1);
    }

    // Set essential callbacks.
    // Reset button.
    {
        let canvas = canvas.clone();
        let ctx = ctx.clone();
        let game = game.clone();

        let f = Closure::wrap(Box::new(move || {
            redraw(&game.borrow(), &canvas, &ctx);
        }) as Box<dyn FnMut()>);

        get_element(&document, "reset-button")?.set_onclick(Some(f.as_ref().unchecked_ref()));
        f.forget();
    }

    // Play button.
    {
        let game = game.clone();

        let f = Closure::wrap(Box::new(move || {
            game.borrow_mut().step();
        }) as Box<dyn FnMut()>);

        get_element(&document, "play-button")?.set_onclick(Some(f.as_ref().unchecked_ref()));
        f.forget();
    }

    // Build main game loop.
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
        }) as Box<dyn FnMut()>));
    }

    // let window = web_sys::window().unwrap();
    // window
    //     .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
    //     .unwrap();

    Ok(())
}

fn get_element(document: &Document, id: &str) -> Result<HtmlElement, Element> {
    document
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlElement>()
}

fn redraw(g: &Game, c: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    ctx.set_fill_style(&JsValue::from_str("white"));
    ctx.fill_rect(0., 0., c.width().into(), c.height().into());

    for (x, y) in g {
        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill_rect(*x as f64 * 10., *y as f64 * 10., 10., 10.);
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
