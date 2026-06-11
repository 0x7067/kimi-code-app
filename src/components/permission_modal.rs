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
                                    onclick: move |_| {
                                        let oid = oid.clone();
                                        // Close the modal only after the backend acknowledges the
                                        // response; on failure, keep it open and surface the error
                                        // so the user can retry instead of silently losing the prompt.
                                        spawn(async move {
                                            match invoke(
                                                "acp_respond_permission",
                                                json!({"requestId": rid, "outcome": {"outcome": "selected", "optionId": oid}}),
                                            ).await {
                                                Ok(_) => *PERMISSION.write() = None,
                                                Err(e) => {
                                                    *ERROR.write() = Some(format!("Failed to send permission response: {e}"));
                                                }
                                            }
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
