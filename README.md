# my-no-sql-node

Read-only replica of [MyNoSqlServer](https://github.com/MyJetTools/my-no-sql-server). A node sits
next to the readers - in another datacenter, for instance - keeps a copy of the tables they
subscribe to and serves them the same way the main node does: SDK TCP readers, HTTP readers and
HTTP reads. Writes always go to the main node.

## How it works

* **Lazy replication.** The node holds only the tables its readers subscribed to. The first reader
  of a table makes the node subscribe to it on the main node; from then on the main node pushes
  every change of the table to the node, and the node pushes it to its readers.
* **Namespaces.** A reader works in the namespace of its connection string
  (`host=node:5125;ns=alpha`), exactly as with the main node. The node opens a connection of its
  own to the main node for every namespace in use - the default one at start up, any other one when
  its first reader subscribes. Namespaces share nothing.
* **Readers are never refused.** A reader which subscribes before the node has connected to the
  main node waits for the table. A table the main node does not have is answered with an empty
  snapshot, just like the main node answers it - never with an error, which the SDK reader panics on.
* **Reconnects.** After a reconnect the node asks the main node for every table again and gets a
  fresh snapshot of each. A table deleted on the main node while the node was away is emptied for
  its readers and asked for again every 30 seconds, so it is picked up once it is created anew.
* **Read statistics.** Last read time and expiration time updates the readers send are forwarded to
  the main node - the node does not expire anything itself.

## Settings

`~/.mynosqlserver-node`:

```yaml
Location: eu-west          # name the node introduces itself with to the main node
MainServer: 10.0.0.1:5125  # reader TCP port of the main node
Compress: true             # the main node compresses what it sends to the node
HttpPort: 5123             # optional, 5123 by default
TcpPort: 5125              # optional, 5125 by default
MaxNamespaces: 16          # optional, 16 by default - a connection to the main node each
```

`MainServer` may be `host:port` or a connection string `host=10.0.0.1:5125`, but must not name a
namespace: the node replicates every namespace its readers subscribe in. `MaxNamespaces` caps how
many: a reader subscribing in one namespace too many is refused with an error, rather than making
the node open connections to the main node without limit.

## Ports

| Port | What |
|---|---|
| `5123` | HTTP: UI (`/`), swagger (`/swagger`), read API, HTTP readers, `/metrics` |
| `5125` | TCP readers - the same protocol as the reader port of the main node |

## HTTP API

The read endpoints of the main node, under the same routes (`/api/...`, the old ones still work):
`/api/Row`, `/api/Count`, `/api/Rows/HighestRowAndBelow`, `/api/Rows/SinglePartitionMultipleRows`,
`/api/Partitions`, `/api/Partitions/Count`, `/api/Tables/List`, `/api/Tables/PartitionsCount`,
`/api/Tables/TableSize`, plus the HTTP reader endpoints `/api/DataReader/*`.

The namespace goes in the `ns` header, or in the `ns` query parameter; none means the default one.
A namespace or a table the node does not hold is answered `400` with `NamespaceNotFound` /
`TableNotFound` - the node holds only what its readers subscribed to.

Monitoring: `/api/IsAlive`, `/api/Status`, `/api/Connections`, `/api/Namespaces/List`,
`/api/Partitions/Details` (filtered by key, 1000 partitions at most).

Everything else on the HTTP port is the UI: its routes (`/`, `/data/...`, `/connections`) and its
files (`/assets/*`, `/favicon.*`). Any other path is a 404.

## UI

`http://node:5123/` - the state of the node as its readers see it:

* **Overview** - the links to the main node (one per namespace) with their latency, the health of
  every reader (how long it has been silent: slow after 6 s, stalled after 15 s - a reader pings
  every ~3 s), which tables are read, and the readers which wait for a table or read one the main
  node does not have.
* **Tables** - the tables of the selected namespace. A node has no tables of its own: each one is
  there because a reader subscribed to it, so this is exactly the set of replicated tables, each
  with its state - replicated, waiting for the main node, or not found on the main node. Partitions
  and rows are read-only.
* **Connections** - live traffic: the table data each link to the main node brings in (its
  uncompressed size - with `Compress` on, far fewer bytes cross the network), what the
  node sends to its readers (TCP readers - HTTP readers are not metered), and the readers of the
  selected namespace.

The UI is a [Dioxus](https://dioxuslabs.com) client-side (wasm) app in `ui/`. It shares its wire
models with the node through the `rest-api-shared` crate, so the two can not disagree on a field.

## Metrics

`/metrics` (Prometheus):

* `main_node_connected{ns}` - `1` while the namespace is connected to the main node;
* `main_node_ping_microseconds{ns}`;
* `table_size{ns,table_name}`, `table_partitions_amount{ns,table_name}`;
* `tcp_connections_count`, `tcp_changes_count{tcp_metric}`, `pending_to_send{table_name}` (by reader).

## Build and release

`build.rs` generates the `Dockerfile` and the GitHub workflows with
[ci-utils](https://github.com/MyJetTools/ci-utils). A release is a tag:

```bash
gh release create 0.3.0 --title "0.3.0" --notes ""
```

The workflow builds the image `ghcr.io/myjettools/my-no-sql-node:<tag>` with the `PUBLISH_TOKEN`
secret. The container runs the binary with `wwwroot` next to it and reads the settings file from the
home folder of its user.

The built UI is committed in `wwwroot/`. After changing anything under `ui/` (or in
`rest-api-shared/`), rebuild it and commit the result:

```bash
./build-ui.sh   # dx build --release --web, then copies the output into wwwroot/
```

It needs the Dioxus CLI of the same minor version as the `dioxus` crate (`dx` 0.7) and the
`wasm32-unknown-unknown` target.
