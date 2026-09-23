pub fn format_bytes(n: f64) -> String {
    let mut n = n;
    if n < 1024.0 {
        return format!("{:.0}b", n);
    }
    n /= 1024.0;
    if n < 1024.0 {
        return format!("{:.2}Kb", n);
    }
    n /= 1024.0;
    if n < 1024.0 {
        return format!("{:.2}Mb", n);
    }
    n /= 1024.0;
    format!("{:.2}Gb", n)
}

/// Throughput that scales with magnitude (b/s, Kb/s, Mb/s, Gb/s) so even a few
/// bytes per second stay visible instead of rounding away to "0.00 MB/s".
pub fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// Round trip to the main node, given in microseconds.
pub fn format_ping(micros: i64) -> String {
    if micros <= 0 {
        return "—".to_string();
    }
    if micros < 1_000 {
        return format!("{}µs", micros);
    }
    format!("{:.2}ms", micros as f64 / 1_000.0)
}
