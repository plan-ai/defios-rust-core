//! Anchor events are used to emit information necessary to
//! index changes made to a SPL ConcurrentMerkleTree

use anchor_lang::prelude::*;

mod application_data;
mod changelog_event;
pub mod job;
pub mod leaf_stake;
pub mod review;

pub use application_data::{ApplicationDataEvent, ApplicationDataEventV1};
pub use changelog_event::{ChangeLogEvent, ChangeLogEventV1};
pub use job::*;
pub use leaf_stake::*;
pub use review::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
#[repr(C)]
pub enum AccountCompressionEvent {
    ChangeLog(ChangeLogEvent),
    ApplicationData(ApplicationDataEvent),
}
