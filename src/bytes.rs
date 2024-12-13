#[macro_export]
macro_rules! format_bytes {
    ($bytes: expr, $prefix: literal) => {{
        let tb = i64::pow(10, 12);
        let gb = i64::pow(10, 9);
        let mb = i64::pow(10, 6);
        let kb = i64::pow(10, 3);

        if $bytes >= tb {
            format!("{:.2} TB{}", $bytes as f32 / tb as f32, $prefix)
        } else if $bytes >= gb {
            format!("{:.2} GB{}", $bytes as f32 / gb as f32, $prefix)
        } else if $bytes >= mb {
            format!("{:.2} MB{}", $bytes as f32 / mb as f32, $prefix)
        } else if $bytes >= kb {
            format!("{:.2} KB{}", $bytes as f32 / kb as f32, $prefix)
        } else {
            format!("{:.2} B{}", $bytes as f32 / mb as f32, $prefix)
        }
    }};

    ($bytes: expr) => {
        format_bytes!($bytes, "")
    };
}
