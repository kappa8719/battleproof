//! Focus management with accessibility

use std::{rc::Rc, sync::Mutex};

use gloo::timers::callback::Timeout;
use sycamore::prelude::*;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{
    AddEventListenerOptions, Element, FocusEvent, HtmlElement, KeyboardEvent, MouseEvent, Node,
    TouchEvent,
};

use crate::{clone, util::dom::events::EventExt};

const CANDIDATE_SELECTOR: &str = "input:not([inert]),\
    select:not([inert]),\
    textarea:not([inert]),\
    a[href]:not([inert]),\
    button:not([inert]),\
    [tabindex]:not(slot):not([inert]),\
    audio[controls]:not([inert]),\
    video[controls]:not([inert]),\
    [contenteditable]:not([contenteditable=\"false\"]):not([inert]),\
    details>summary:first-of-type:not([inert]),\
    details:not([inert])";

fn is_disabled(element: &Element) -> bool {
    if let Some(disabled) = element.get_attribute("disabled") {
        return match disabled.as_str() {
            "true" => true,
            &_ => false,
        };
    }

    false
}

fn is_inert(element: &Element) -> bool {
    if let Some(inert) = element.get_attribute("inert") {
        return inert == "true";
    }

    false
}

fn has_inert_ancestor(element: &Element) -> bool {
    let Some(mut current) = element.parent_element() else {
        return false;
    };
    loop {
        if is_inert(element) {
            return true;
        }

        if let Some(parent) = current.parent_element() {
            current = parent;
        } else {
            return false;
        }
    }
}

fn is_hidden_input(element: &Element) -> bool {
    if element.tag_name() == "input"
        && let Some(t) = element.get_attribute("type")
        && t == "hidden"
    {
        return true;
    }

    false
}

pub fn is_focusable(element: &HtmlElement) -> bool {
    if is_disabled(element)
        || is_inert(element)
        || has_inert_ancestor(element)
        || is_hidden_input(element)
    {
        return false;
    }

    true
}

pub fn is_tabbable(element: &HtmlElement) -> bool {
    if element.tab_index() < 0 || !is_focusable(element) {
        return false;
    }

    true
}

fn candidates(container: &Element, filter: impl Fn(&HtmlElement) -> bool) -> Vec<HtmlElement> {
    let Ok(elements) = container.query_selector_all(CANDIDATE_SELECTOR) else {
        return vec![];
    };

    let mut candidates = Vec::with_capacity(elements.length() as usize);
    for element in elements.values() {
        let Ok(element) = element else {
            continue;
        };

        let Ok(element) = element.dyn_into::<HtmlElement>() else {
            continue;
        };

        if !filter(&element) {
            continue;
        }

        candidates.push(element);
    }

    candidates
}

fn first_candidate(
    container: &Element,
    filter: impl Fn(&HtmlElement) -> bool,
) -> Option<HtmlElement> {
    let Ok(elements) = container.query_selector_all(CANDIDATE_SELECTOR) else {
        return None;
    };

    for element in elements.values() {
        let Ok(element) = element else {
            continue;
        };

        let Ok(element) = element.dyn_into::<HtmlElement>() else {
            continue;
        };

        if filter(&element) {
            return Some(element);
        }
    }

    None
}

pub fn tab_candidates(container: &Element) -> Vec<HtmlElement> {
    candidates(container, is_tabbable)
}

pub fn focus_candidates(container: &Element) -> Vec<HtmlElement> {
    candidates(container, is_focusable)
}

struct FocusTrapListeners {
    focus_in: Closure<dyn Fn(FocusEvent)>,
    mouse_down: Closure<dyn Fn(MouseEvent)>,
    touch_start: Closure<dyn Fn(TouchEvent)>,
    click: Closure<dyn Fn(MouseEvent)>,
    key_down: Closure<dyn Fn(KeyboardEvent)>,
}

pub struct FocusTrap {
    /// Attachted event listeners
    listeners: Option<FocusTrapListeners>,
    /// The last focused node
    last_focus: Option<Node>,
    /// The target container this focus trap is initialized
    container: Element,
}

impl FocusTrap {
    pub fn activate(this: Rc<Mutex<Self>>) {
        {
            let this = this.lock().unwrap();
            if this.should_focus_initially() {
                this.focus_initially();
            }
        }

        Self::add_listeners(this);
    }

    pub fn deactivate(this: Rc<Mutex<Self>>) {
        Self::remove_listeners(this);
    }

    fn add_listeners(this: Rc<Mutex<Self>>) {
        if this.lock().unwrap().listeners.is_some() {
            return;
        }

        let document = document();

        let listener_focus_in: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            let mut this = this.lock().expect("failed to acquire lock");
            this.handle_focus_in(event);
        }));
        let listener_mouse_down: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            let mut this = this.lock().expect("failed to acquire lock");
            this.handle_mouse_down(event);
        }));
        let listener_touch_start: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            let mut this = this.lock().expect("failed to acquire lock");
            this.handle_touch_start(event);
        }));
        let listener_click: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            let mut this = this.lock().expect("failed to acquire lock");
            this.handle_click(event);
        }));
        let listener_key_down: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            let mut this = this.lock().expect("failed to acquire lock");
            this.handle_key_down(event);
        }));

        let options_capture = {
            let v = AddEventListenerOptions::new();
            v.set_capture(true);
            v
        };

        let _ = document.add_event_listener_with_callback_and_bool(
            "focusin",
            listener_focus_in.as_ref().unchecked_ref(),
            true,
        );
        let _ = document.add_event_listener_with_callback_and_add_event_listener_options(
            "mousedown",
            listener_mouse_down.as_ref().unchecked_ref(),
            &options_capture,
        );
        let _ = document.add_event_listener_with_callback_and_add_event_listener_options(
            "touchstart",
            listener_touch_start.as_ref().unchecked_ref(),
            &options_capture,
        );
        let _ = document.add_event_listener_with_callback_and_add_event_listener_options(
            "click",
            listener_click.as_ref().unchecked_ref(),
            &options_capture,
        );
        let _ = document.add_event_listener_with_callback_and_add_event_listener_options(
            "keydown",
            listener_key_down.as_ref().unchecked_ref(),
            &options_capture,
        );
        let _ = document.add_event_listener_with_callback(
            "keydown",
            listener_key_down.as_ref().unchecked_ref(),
        );

        let listeners = FocusTrapListeners {
            focus_in: listener_focus_in,
            mouse_down: listener_mouse_down,
            touch_start: listener_touch_start,
            click: listener_click,
            key_down: listener_key_down,
        };

        let mut this = this.lock().unwrap();
        this.listeners = Some(listeners);
    }

    fn remove_listeners(this: Rc<Mutex<Self>>) {
        let mut this = this.lock().unwrap();
        let Some(listeners) = this.listeners.take() else {
            return;
        };

        let document = document();
        let _ = document.remove_event_listener_with_callback_and_bool(
            "focusin",
            listeners.focus_in.as_ref().unchecked_ref(),
            true,
        );
        let _ = document.remove_event_listener_with_callback(
            "mousedown",
            listeners.mouse_down.as_ref().unchecked_ref(),
        );
        let _ = document.remove_event_listener_with_callback(
            "touchstart",
            listeners.touch_start.as_ref().unchecked_ref(),
        );
        let _ = document
            .remove_event_listener_with_callback("click", listeners.click.as_ref().unchecked_ref());
        let _ = document.remove_event_listener_with_callback(
            "keydown",
            listeners.key_down.as_ref().unchecked_ref(),
        );

        listeners.focus_in.forget();
        listeners.mouse_down.forget();
        listeners.touch_start.forget();
        listeners.click.forget();
        listeners.key_down.forget();
    }

    fn get_tab_candidates(&self) -> Vec<HtmlElement> {
        tab_candidates(&self.container)
    }

    fn get_focus_candidates(&self) -> Vec<HtmlElement> {
        focus_candidates(&self.container)
    }

    fn get_first_tab_candidate(&self) -> Option<HtmlElement> {
        first_candidate(&self.container, is_tabbable)
    }

    fn get_first_focus_candidate(&self) -> Option<HtmlElement> {
        first_candidate(&self.container, is_focusable)
    }

    fn should_focus_initially(&self) -> bool {
        let Some(current) = document().active_element() else {
            return false;
        };

        if self.container.contains(Some(&current)) {
            return false;
        }

        true
    }

    fn focus_initially(&self) {
        let Some(candidate) = self.get_first_focus_candidate() else {
            return;
        };

        Self::focus_asynchronously(candidate);
    }

    fn focus_asynchronously(element: HtmlElement) {
        Timeout::new(0, move || {
            let _ = element.focus();
        })
        .forget();
    }

    fn handle_focus_in(&mut self, event: FocusEvent) {
        let Some(target) = event.actual_target() else {
            return;
        };

        let Ok(target) = target.dyn_into::<Node>() else {
            return;
        };

        if self.container.contains(Some(&target)) {
            self.last_focus = Some(target)
        } else {
            // the focus has escaped out of focus trap
            event.stop_immediate_propagation();

            if let Some(last_focus) = &self.last_focus
                && let Ok(last_focus) = last_focus.clone().dyn_into::<HtmlElement>()
            {
                Self::focus_asynchronously(last_focus);
            }
        }
    }

    fn handle_mouse_down(&self, event: MouseEvent) {}

    fn handle_touch_start(&self, event: TouchEvent) {}

    fn handle_click(&self, event: MouseEvent) {}

    fn handle_key_down(&self, event: KeyboardEvent) {}
}

pub fn create_focus_trap(container: Element) -> Rc<Mutex<FocusTrap>> {
    let trap = FocusTrap {
        listeners: None,
        last_focus: None,
        container,
    };

    Rc::new(Mutex::new(trap))
}
