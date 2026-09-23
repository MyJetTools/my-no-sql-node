use std::time::Duration;

use dioxus::prelude::*;

mod api;
mod components;
mod models;
mod pages;
mod route_key;
mod settings;
mod storage;
mod utils;

use components::shell::{Crumb, NamespaceOption, Sidebar, SidebarSection, Topbar};
use models::{DEFAULT_NAMESPACE, StatusModel};
use pages::*;
use route_key::RouteKey;
use settings::HealthThresholds;

/// How often the shell re-reads `/api/Status` - every page renders from it.
const STATUS_REFRESH: Duration = Duration::from_secs(1);

#[derive(Routable, PartialEq, Clone)]
pub enum AppRoute {
    #[layout(Shell)]
    #[route("/")]
    Home {},
    #[layout(DataLayout)]
    #[route("/data")]
    Data {},
    #[route("/data/:table")]
    DataTable { table: RouteKey },
    #[route("/data/:table/:partition")]
    DataPartition { table: RouteKey, partition: RouteKey },
    #[route("/data/:table/:partition/:row")]
    DataRow {
        table: RouteKey,
        partition: RouteKey,
        row: RouteKey,
    },
    #[end_layout]
    #[route("/connections")]
    Connections {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

/// What every page of the UI renders from: the node's status, polled by the shell.
#[derive(Clone, Default)]
pub struct AppContext {
    /// The last status the node answered with. Kept when a poll fails: one lost request must
    /// not blank every page - `status_error` says the data is stale.
    pub status: Option<StatusModel>,
    /// Why the last poll failed. Cleared by the next successful one.
    pub status_error: Option<String>,
}

impl AppContext {
    fn set_status(&mut self, status: StatusModel) {
        self.status = Some(status);
        self.status_error = None;
    }

    fn set_status_error(&mut self, err: String) {
        self.status_error = Some(err);
    }

    /// The status is fresh - the last poll succeeded.
    pub fn is_online(&self) -> bool {
        self.status.is_some() && self.status_error.is_none()
    }
}

fn main() {
    dioxus::LaunchBuilder::new().launch(|| {
        let theme = storage::load_theme().unwrap_or_else(|| "light".to_string());
        storage::apply_theme(&theme);

        rsx! {
            // The SVG is the real icon; the .ico stays as the fallback for the
            // browsers that ignore an SVG favicon.
            document::Link {
                rel: "icon",
                r#type: "image/svg+xml",
                href: asset!("/public/favicon.svg"),
            }
            document::Link {
                rel: "alternate icon",
                r#type: "image/x-icon",
                href: asset!("/public/favicon.ico"),
            }
            Router::<AppRoute> {}
        }
    });
}

#[component]
fn Shell() -> Element {
    let mut ctx = use_context_provider(|| Signal::new(AppContext::default()));

    // The node has no settings storage - the thresholds are fixed.
    use_context_provider(|| Signal::new(HealthThresholds::default()));

    // One poller for the whole UI, alive as long as the shell is.
    use_hook(move || {
        spawn(async move {
            let mut first = true;
            loop {
                match crate::api::get_status().await {
                    Ok(status) => {
                        if first && !is_selected_namespace_known(&status) {
                            // A namespace remembered from an earlier session which the node
                            // does not replicate (any more): everything below would be empty.
                            storage::save_namespace("");
                            reload_into_root();
                            return;
                        }
                        first = false;
                        ctx.write().set_status(status);
                    }
                    Err(err) => {
                        dioxus_utils::console_log(format!("Status error: {}", err.details));
                        ctx.write().set_status_error(err.message);
                    }
                }
                dioxus_utils::js::sleep(STATUS_REFRESH).await;
            }
        });
    });

    let current_ns = storage::load_namespace().unwrap_or_default();
    // `current_ns` is the value of the namespace select, where an empty string is the default
    // namespace. The status names it for real.
    let selected_ns_name = if current_ns.is_empty() {
        DEFAULT_NAMESPACE.to_string()
    } else {
        current_ns.clone()
    };

    let on_namespace_change = move |namespace: String| {
        if namespace == storage::load_namespace().unwrap_or_default() {
            return;
        }

        storage::save_namespace(namespace.as_str());

        // A hard navigation to the root, not a re-render: the table, partition and row sitting
        // in the URL belong to the previous namespace.
        reload_into_root();
    };

    let route = use_route::<AppRoute>();
    let section = match &route {
        AppRoute::Home {} | AppRoute::NotFound { .. } => SidebarSection::Overview,
        AppRoute::Data {}
        | AppRoute::DataTable { .. }
        | AppRoute::DataPartition { .. }
        | AppRoute::DataRow { .. } => SidebarSection::Tables,
        AppRoute::Connections {} => SidebarSection::Connections,
    };

    let crumbs = build_crumbs(&route);

    let ctx_ra = ctx.read();
    let status = ctx_ra.status.as_ref();

    let online = ctx_ra.is_online();
    let version = status
        .map(|s| s.status_bar.version.clone())
        .unwrap_or_default();
    let location = status
        .map(|s| s.status_bar.location.id.clone())
        .unwrap_or_default();

    let (tables_in_current_ns, readers_in_current_ns, readers_count) = match status {
        Some(s) => (
            s.initialized
                .tables
                .iter()
                .filter(|t| t.namespace == selected_ns_name)
                .count(),
            s.initialized
                .readers
                .iter()
                .filter(|r| r.namespace == selected_ns_name)
                .count(),
            s.initialized.readers.len(),
        ),
        None => (0, 0, 0),
    };

    // Readers grouped by the namespace they work in, biggest first.
    let readers_by_namespace: Vec<(String, usize)> = match status {
        Some(s) => {
            let mut by_namespace: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();

            for reader in s.initialized.readers.iter() {
                *by_namespace.entry(reader.namespace.clone()).or_default() += 1;
            }

            let mut result: Vec<(String, usize)> = by_namespace.into_iter().collect();
            result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            result
        }
        None => Vec::new(),
    };

    let namespaces: Vec<NamespaceOption> = match status {
        Some(s) => s
            .initialized
            .namespaces
            .iter()
            .map(|ns| NamespaceOption {
                name: ns.name.clone(),
                tables_amount: s
                    .initialized
                    .tables
                    .iter()
                    .filter(|t| t.namespace == ns.name)
                    .count(),
                connected: ns.connected_to_main_node,
            })
            .collect(),
        None => Vec::new(),
    };
    drop(ctx_ra);

    rsx! {
        div { class: "shell",
            Sidebar {
                active: section,
                version,
                location,
                tables_count: tables_in_current_ns,
                readers_count,
                readers_in_current_ns,
                readers_by_namespace,
                online,
            }
            div { class: "main",
                Topbar {
                    crumbs,
                    namespaces,
                    current_ns,
                    on_namespace_change,
                }
                Outlet::<AppRoute> {}
            }
        }
    }
}

/// `false` only for a remembered namespace the node does not replicate.
fn is_selected_namespace_known(status: &StatusModel) -> bool {
    let Some(selected) = storage::load_namespace() else {
        return true;
    };

    status
        .initialized
        .namespaces
        .iter()
        .any(|itm| itm.name == selected)
}

/// Sends the browser to the app root and reloads it from scratch.
fn reload_into_root() {
    let _ = dioxus::document::eval("window.location.href = '/';");
}

fn crumb(label: &str, active: bool) -> Crumb {
    Crumb {
        label: label.to_string(),
        active,
    }
}

fn build_crumbs(route: &AppRoute) -> Vec<Crumb> {
    match route {
        AppRoute::Home {} | AppRoute::NotFound { .. } => {
            vec![crumb("MyNoSql Node", false), crumb("Overview", true)]
        }
        AppRoute::Connections {} => {
            vec![crumb("MyNoSql Node", false), crumb("Connections", true)]
        }
        AppRoute::Data {}
        | AppRoute::DataTable { .. }
        | AppRoute::DataPartition { .. }
        | AppRoute::DataRow { .. } => build_data_crumbs(route),
    }
}

/// Breadcrumbs of the data routes — reflects the `/data/<table>/<partition>/<row>` path, with
/// the deepest selected segment marked active.
fn build_data_crumbs(route: &AppRoute) -> Vec<Crumb> {
    let (table, partition, row) = match route {
        AppRoute::DataTable { table } => (Some(table.as_str()), None, None),
        AppRoute::DataPartition { table, partition } => {
            (Some(table.as_str()), Some(partition.as_str()), None)
        }
        AppRoute::DataRow {
            table,
            partition,
            row,
        } => (
            Some(table.as_str()),
            Some(partition.as_str()),
            Some(row.as_str()),
        ),
        AppRoute::Home {}
        | AppRoute::Data {}
        | AppRoute::Connections {}
        | AppRoute::NotFound { .. } => (None, None, None),
    };

    let mut crumbs = vec![crumb("MyNoSql Node", false), crumb("Tables", table.is_none())];
    if let Some(t) = table {
        crumbs.push(crumb(t, partition.is_none()));
    }
    if let Some(p) = partition {
        crumbs.push(crumb(p, row.is_none()));
    }
    if let Some(r) = row {
        crumbs.push(crumb(r, true));
    }
    crumbs
}
