pub fn byte_count_to_string_with_binary_prefix(byte_count: u64) -> String {
    const BINARY_PREFIXES: [&str; 4] = ["", "Ki", "Mi", "Gi"];

    if byte_count < 1024 {
        return format!("{byte_count} B");
    }

    let mut byte_count_with_prefix = byte_count as f64;
    let mut prefix_index = 0;
    while byte_count_with_prefix as u64 >= 1024 && prefix_index + 1 < BINARY_PREFIXES.len() {
        byte_count_with_prefix /= 1024.0;

        prefix_index += 1;
    }

    format!("{:.2} {}B", byte_count_with_prefix, BINARY_PREFIXES[prefix_index])
}
