use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::json;

#[component]
pub fn PermissionModal() -> Element {
    let Some(req) = PERMISSION.read().clone() else {
        return rsx! {};
    };
    let mut responding = use_signal(|| None::<String>);
    rsx! {
        div { class: "overlay", "data-testid": "permission-overlay",
            div { class: "modal permission-modal", "data-testid": "permission-modal",
                div { class: "permission-head",
                    span { class: "permission-badge", "Approval" }
                    h3 { "Permission required" }
                }
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
                            let busy = responding.read().as_ref() == Some(&option_id);
                            let disabled = responding.read().is_some();
                            rsx! {
                                button {
                                    "data-testid": "permission-option",
                                    "data-option-id": "{option_id}",
                                    key: "{option_id}",
                                    class: "{class}",
                                    disabled,
                                    onclick: move |_| {
                                        let oid = oid.clone();
                                        responding.set(Some(oid.clone()));
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
                                                    responding.set(None);
                                                    *ERROR.write() = Some(format!("Failed to send permission response: {e}"));
                                                }
                                            }
                                        });
                                    },
                                    if busy { "Sending…" } else { "{name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
