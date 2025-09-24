//! Assign and retrieve id for elements

use std::time::{SystemTime, UNIX_EPOCH};

use snowflake::{Snowflake, SnowflakeBuilder};
use web_sys::HtmlElement;

static mut SNOWFLAKE: Snowflake = Snowflake::new(0, 0);

pub fn initialize() {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    unsafe {
        SNOWFLAKE = SnowflakeBuilder::new().epoch(time).build();
    }
}

fn snowflake() -> &'static Snowflake {
    #[allow(static_mut_refs)]
    unsafe {
        &SNOWFLAKE
    }
}

pub fn read_id_for_element(element: HtmlElement) -> u64 {
    if element.has_attribute("data-uid")
        && let Some(uid) = element.get_attribute("data-uid")
        && let Ok(uid) = uid.parse::<u64>()
    {
        return uid;
    }

    let uid = snowflake().next_id();
    let _ = element.set_attribute("data-uid", uid.to_string().as_str());

    uid
}

pub fn dispose_id_for_element(element: HtmlElement) {
    let _ = element.remove_attribute("data-uid");
}
