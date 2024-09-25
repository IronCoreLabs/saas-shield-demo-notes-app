pub mod components;
pub mod pages;
pub mod util;

use sycamore::prelude::*;

use crate::pages::index::Index;

fn main() {
    sycamore::render(|| view! { Index {} });
}
