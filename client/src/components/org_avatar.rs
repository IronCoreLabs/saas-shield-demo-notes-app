use super::atoms::office_icon::OfficeIcon;
use sycamore::prelude::*;

#[component(inline_props)]
pub fn OrgAvatar(org_name: String) -> View {
    view! {
        div(class="p-1 mx-auto flex justify-start space-x-2 text-white w-full") {
            div(class="relative w-7 h-7 rounded-full".to_string()) {
                OfficeIcon(class="absolute w-8 h-8 -left-1".to_string())
            }
            div(class="flex items-center") {
                p(class="text-sm") { (org_name) }
            }
        }
    }
}
