/// Bytes a connection received and sent during the last second.
#[derive(Debug, Default, Clone, Copy)]
pub struct TrafficPerSecond {
    pub incoming: usize,
    pub outgoing: usize,
}
