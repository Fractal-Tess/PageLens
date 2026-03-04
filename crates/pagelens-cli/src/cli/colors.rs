//! Color parsing and contrast calculations

pub fn parse_rgb_triplet(value: &str) -> Option<(u8, u8, u8)> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    if trimmed.eq_ignore_ascii_case("transparent") {
        return None;
    }
    let body = if trimmed.starts_with("rgb(") {
        trimmed.strip_prefix("rgb(")?.strip_suffix(')')?
    } else if trimmed.starts_with("rgba(") {
        trimmed.strip_prefix("rgba(")?.strip_suffix(')')?
    } else {
        return None;
    };

    let parts = body.split(',').map(|p| p.trim()).collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }

    let r = parse_rgb_component(parts[0])?;
    let g = parse_rgb_component(parts[1])?;
    let b = parse_rgb_component(parts[2])?;
    Some((r, g, b))
}

pub fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some((r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        8 => {
            let alpha = u8::from_str_radix(&hex[6..8], 16).ok()?;
            if alpha == 0 {
                return None;
            }
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

pub fn parse_rgb_component(part: &str) -> Option<u8> {
    if let Some(percent) = part.strip_suffix('%') {
        let p = percent.trim().parse::<f64>().ok()?;
        let clamped = p.clamp(0.0, 100.0);
        return Some(((clamped / 100.0) * 255.0).round() as u8);
    }
    let n = part.trim().parse::<f64>().ok()?;
    Some(n.clamp(0.0, 255.0).round() as u8)
}

pub fn contrast_ratio_from_luminance(la: f64, lb: f64) -> f64 {
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

pub fn color_luminance(value: &str) -> Option<f64> {
    if let Some(rgb) = parse_rgb_triplet(value) {
        return Some(relative_luminance(rgb));
    }
    parse_lab_luminance(value)
        .or_else(|| parse_oklab_luminance(value))
        .or_else(|| parse_oklch_luminance(value))
}

pub fn parse_lab_luminance(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if !trimmed.starts_with("lab(") {
        return None;
    }
    let body = trimmed.strip_prefix("lab(")?.strip_suffix(')')?;
    let mut parts = body.split('/');
    let left = parts.next()?.trim();
    if let Some(alpha_raw) = parts.next() {
        let alpha = parse_alpha(alpha_raw.trim())?;
        if alpha <= 0.0 {
            return None;
        }
    }

    let lightness_raw = left.split_whitespace().next()?;
    let lightness = if let Some(percent) = lightness_raw.strip_suffix('%') {
        percent.trim().parse::<f64>().ok()?
    } else {
        lightness_raw.parse::<f64>().ok()?
    };
    let l = lightness.clamp(0.0, 100.0);

    let y = if l > 8.0 {
        ((l + 16.0) / 116.0).powf(3.0)
    } else {
        l / 903.3
    };
    Some(y.clamp(0.0, 1.0))
}

pub fn parse_oklab_luminance(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if !trimmed.starts_with("oklab(") {
        return None;
    }
    let body = trimmed.strip_prefix("oklab(")?.strip_suffix(')')?;
    let mut parts = body.split('/');
    let left = parts.next()?.trim();
    if let Some(alpha_raw) = parts.next() {
        let alpha = parse_alpha(alpha_raw.trim())?;
        if alpha <= 0.0 {
            return None;
        }
    }

    let components = left.split_whitespace().collect::<Vec<_>>();
    if components.len() < 3 {
        return None;
    }
    let l = parse_percent_or_number(components[0], 1.0)?;
    let a = parse_percent_or_number(components[1], 0.4)?;
    let b = parse_percent_or_number(components[2], 0.4)?;
    oklab_luminance(l, a, b)
}

pub fn parse_oklch_luminance(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if !trimmed.starts_with("oklch(") {
        return None;
    }
    let body = trimmed.strip_prefix("oklch(")?.strip_suffix(')')?;
    let mut parts = body.split('/');
    let left = parts.next()?.trim();
    if let Some(alpha_raw) = parts.next() {
        let alpha = parse_alpha(alpha_raw.trim())?;
        if alpha <= 0.0 {
            return None;
        }
    }

    let components = left.split_whitespace().collect::<Vec<_>>();
    if components.len() < 3 {
        return None;
    }
    let l = parse_percent_or_number(components[0], 1.0)?;
    let c = parse_percent_or_number(components[1], 0.4)?;
    let h = components[2]
        .trim()
        .trim_end_matches("deg")
        .parse::<f64>()
        .ok()?;
    let radians = h.to_radians();
    let a = c * radians.cos();
    let b = c * radians.sin();
    oklab_luminance(l, a, b)
}

pub fn parse_percent_or_number(raw: &str, percent_scale: f64) -> Option<f64> {
    let trimmed = raw.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        let p = percent.trim().parse::<f64>().ok()?;
        return Some((p / 100.0) * percent_scale);
    }
    trimmed.parse::<f64>().ok()
}

pub fn oklab_luminance(l: f64, a: f64, b: f64) -> Option<f64> {
    let l_ = l + (0.3963377774 * a) + (0.2158037573 * b);
    let m_ = l - (0.1055613458 * a) - (0.0638541728 * b);
    let s_ = l - (0.0894841775 * a) - (1.2914855480 * b);

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r = (4.0767416621 * l3) - (3.3077115913 * m3) + (0.2309699292 * s3);
    let g = (-1.2684380046 * l3) + (2.6097574011 * m3) - (0.3413193965 * s3);
    let b_ = (-0.0041960863 * l3) - (0.7034186147 * m3) + (1.7076147010 * s3);

    if !r.is_finite() || !g.is_finite() || !b_.is_finite() {
        return None;
    }

    let y = (0.2126 * r) + (0.7152 * g) + (0.0722 * b_);
    Some(y.clamp(0.0, 1.0))
}

pub fn parse_alpha(alpha_raw: &str) -> Option<f64> {
    let trimmed = alpha_raw.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        let p = percent.trim().parse::<f64>().ok()?;
        return Some((p / 100.0).clamp(0.0, 1.0));
    }
    let value = trimmed.parse::<f64>().ok()?;
    Some(value.clamp(0.0, 1.0))
}

pub fn relative_luminance((r, g, b): (u8, u8, u8)) -> f64 {
    fn channel(v: u8) -> f64 {
        let s = (v as f64) / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    (0.2126 * channel(r)) + (0.7152 * channel(g)) + (0.0722 * channel(b))
}
