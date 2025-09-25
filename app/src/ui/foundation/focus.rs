//! Focus management with accessibility

use std::sync::Arc;

use sycamore::prelude::*;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{AddEventListenerOptions, FocusEvent, KeyboardEvent, MouseEvent, TouchEvent};

use crate::clone;

pub struct FocusTrap {}

impl FocusTrap {
    fn add_listeners(this: Arc<FocusTrap>) {
        let document = document();

        let listener_focus_in: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            this.handle_focus_in(event);
        }));
        let listener_mouse_down: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            this.handle_mouse_down(event);
        }));
        let listener_touch_start: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            this.handle_touch_start(event);
        }));
        let listener_click: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
            this.handle_click(event);
        }));
        let listener_key_down: Closure<dyn Fn(_)> = Closure::new(clone!([this], move |event| {
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
    }

    fn handle_focus_in(&self, event: FocusEvent) {}

    fn handle_mouse_down(&self, event: MouseEvent) {}

    fn handle_touch_start(&self, event: TouchEvent) {}

    fn handle_click(&self, event: MouseEvent) {}

    fn handle_key_down(&self, event: KeyboardEvent) {}
}

pub fn create_focus_trap() {}
