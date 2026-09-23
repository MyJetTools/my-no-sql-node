use std::cell::OnceCell;

const THEME_KEY: &str = "mns_theme";

/// Namespace the UI is pointed at. Absent means the default namespace, which is what the node
/// assumes when a request carries no `ns` header.
const NAMESPACE_KEY: &str = "mns_namespace";

thread_local! {
    /// The namespace this tab works in, read once. localStorage is shared by every tab of the
    /// origin: re-reading it would let a switch made in another tab silently re-point this one,
    /// with the table and partition of the old namespace still in its URL.
    static NAMESPACE: OnceCell<Option<String>> = const { OnceCell::new() };
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn read_namespace() -> Option<String> {
    let s = local_storage()?;
    s.get_item(NAMESPACE_KEY).ok()?.filter(|v| !v.is_empty())
}

/// The namespace this tab works in - `None` is the default one. Fixed for the life of the page.
pub fn load_namespace() -> Option<String> {
    NAMESPACE.with(|itm| itm.get_or_init(read_namespace).clone())
}

/// Remembers the namespace for the next page load. The UI reloads right after it - see
/// `load_namespace`.
pub fn save_namespace(namespace: &str) {
    if let Some(s) = local_storage() {
        // The default namespace is stored as "nothing selected": the UI then sends no header.
        if namespace.is_empty() {
            let _ = s.remove_item(NAMESPACE_KEY);
        } else {
            let _ = s.set_item(NAMESPACE_KEY, namespace);
        }
    }
}

pub fn save_theme(theme: &str) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(THEME_KEY, theme);
    }
}

pub fn load_theme() -> Option<String> {
    let s = local_storage()?;
    s.get_item(THEME_KEY).ok()?.filter(|v| !v.is_empty())
}

pub fn apply_theme(theme: &str) {
    if let Some(window) = web_sys::window()
        && let Some(doc) = window.document()
        && let Some(html) = doc.document_element()
    {
        let _ = html.set_attribute("data-theme", theme);
    }
}
