/// A reader pings about every 3 seconds, so a live one is often silent for a bit over 3 -
/// "slow" starts well above that, with room for jitter.
pub const DEFAULT_WARN_MS: u32 = 6_000;
pub const DEFAULT_BAD_MS: u32 = 15_000;

/// Health thresholds for the reader status indicator: how long a reader may stay silent before
/// it is shown Yellow (`warn_ms`) and Red (`bad_ms`). The node has no settings storage, so these
/// are fixed.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct HealthThresholds {
    pub warn_ms: u32,
    pub bad_ms: u32,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            warn_ms: DEFAULT_WARN_MS,
            bad_ms: DEFAULT_BAD_MS,
        }
    }
}
