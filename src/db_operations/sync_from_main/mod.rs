//! Changes the main node pushes to a namespace of this node.
//!
//! Data of a table the node does not hold yet is skipped: the table is on its way, and the
//! snapshot which is going to bring it is taken after every change sent before it, so it
//! carries them all.

mod delete_rows;
mod report_broken_payload;
mod sync_partition;
mod sync_rows;
mod sync_table;
mod table_not_found;
pub use delete_rows::*;
use report_broken_payload::*;
pub use sync_partition::*;
pub use sync_rows::*;
pub use sync_table::*;
pub use table_not_found::*;
