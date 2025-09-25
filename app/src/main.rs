mod ui;
mod util;

use sycamore::prelude::*;

fn main() {
    console_error_panic_hook::set_once();

    ui::foundation::id::initialize();

    sycamore::render(|| {
        view! {
            button(class = "btn btn-primary") { "Button" }
        }
    });
}
