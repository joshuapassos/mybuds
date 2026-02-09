/// Embedded 64x64 icon PNG.
const ICON_PNG: &[u8] = include_bytes!("../../assets/icon-64.png");

/// Decode the embedded PNG and return ARGB32 pixel data for ksni.
/// ksni expects each pixel as [A, R, G, B] (network byte order).
pub fn tray_icon() -> (i32, i32, Vec<u8>) {
    let img = image::load_from_memory(ICON_PNG)
        .expect("embedded icon PNG is valid")
        .to_rgba8();
    let (w, h) = (img.width(), img.height());

    // Convert RGBA → ARGB
    let mut argb = Vec::with_capacity((w * h * 4) as usize);
    for pixel in img.pixels() {
        let [r, g, b, a] = pixel.0;
        argb.extend_from_slice(&[a, r, g, b]);
    }

    (w as i32, h as i32, argb)
}
