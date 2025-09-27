use wasm_bindgen::JsCast;
use web_sys::{Element, Event, EventTarget};

pub trait EventExt {
    fn actual_target(&self) -> Option<EventTarget>;
}

impl EventExt for Event {
    fn actual_target(&self) -> Option<EventTarget> {
        let target = self.target()?;
        let element = target.dyn_into::<Element>().ok()?;
        if element.shadow_root().is_some() {
            let composed_path = self.composed_path();
            if composed_path.length() > 0 {
                return composed_path.get(0).dyn_into::<EventTarget>().ok();
            }
        }

        self.target()
    }
}
