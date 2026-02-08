/// Generate a simple tray icon as an ARGB pixel buffer.
/// ksni expects ARGB32 in network byte order: [A, R, G, B] per pixel.
pub fn generate_tray_icon(size: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (size * size * 4) as usize];

    let s = size as f32;
    let cx = s / 2.0;
    let cy = s / 2.0;

    // Headband arc (top half of circle)
    let r = s * 0.35;
    for y in 0..size {
        for x in 0..size {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let dx = fx - cx;
            let dy = fy - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = ((y * size + x) * 4) as usize;

            // Headband (top arc)
            if dist > r - 1.5 && dist < r + 1.5 && fy < cy + 2.0 {
                pixels[idx] = 0xFF;     // A
                pixels[idx + 1] = 0xFF; // R
                pixels[idx + 2] = 0xFF; // G
                pixels[idx + 3] = 0xFF; // B
            }

            // Left ear cup
            let ear_w = s * 0.15;
            let ear_h = s * 0.25;
            let ear_lx = cx - r;
            let ear_ly = cy;
            if fx > ear_lx - ear_w / 2.0
                && fx < ear_lx + ear_w / 2.0
                && fy > ear_ly
                && fy < ear_ly + ear_h
            {
                pixels[idx] = 0xFF;
                pixels[idx + 1] = 0xFF;
                pixels[idx + 2] = 0xFF;
                pixels[idx + 3] = 0xFF;
            }

            // Right ear cup
            let ear_rx = cx + r;
            if fx > ear_rx - ear_w / 2.0
                && fx < ear_rx + ear_w / 2.0
                && fy > ear_ly
                && fy < ear_ly + ear_h
            {
                pixels[idx] = 0xFF;
                pixels[idx + 1] = 0xFF;
                pixels[idx + 2] = 0xFF;
                pixels[idx + 3] = 0xFF;
            }
        }
    }

    pixels
}
