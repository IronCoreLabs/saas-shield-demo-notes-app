pub mod apis;
pub mod components;
pub mod pages;
pub mod util;

use crate::pages::index::Index;
use sycamore::prelude::*;

fn main() {
    sycamore::render(|| view! { Index {} });
}
