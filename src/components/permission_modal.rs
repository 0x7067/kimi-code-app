use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::json;

#[component]
pub fn PermissionModal() -> Element {
    let Some(req) = PERMISSION.read().clone() else { return rsx! {} };
    rsx! {
        div { class: "overlay",
            div { class: "modal",
                h3 { "Permission required" }
                p { class: "perm-title", "{req.title}" }
                if !req.detail.is_empty() {
                    pre { class: "perm-detail", "{req.detail}" }
                }
                div { class: "modal-actions",
                    for (option_id, name, kind) in req.options.clone() {
                        {
                            let rid = req.request_id;
                            let oid = option_id.clone();
                            let class = if kind.starts_with("allow") { "primary" } else { "ghost" };
                            rsx! {
                                button {
                                    key: "{option_id}",
                                    class: "{class}",
                                    onclick: |_| {
                                        let oid = oid.clone();
                                        *PERMISSION.write() = None;
                                        spawn(async move {
                                            let _ = invoke(
                                                "acp_respond_permission",
                                                json!({"requestId": rid, "outcome": {"outcome": "selected", "optionId": oid}}),
                                            ).await;
                                        });
                                    },
                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
