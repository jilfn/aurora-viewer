//! Inventory thumbnails: lossless JPEG2000, as LLImageJ2COJ::encodeImpl.
//! Encoded by openjp2, the Rust port of OpenJPEG: its parameters are OpenJPEG's.

use super::*;

/// Square, power-of-two RGB/RGBA thumbnails, 64–256 px (LLFloaterSimpleSnapshot).
pub fn encode_thumbnail_j2k(width: u32, height: u32, components: u8, data: &[u8]) -> Result<Vec<u8>, AssetError> {
    if width != height
        || !(64..=256).contains(&width)
        || !width.is_power_of_two()
        || !matches!(components, 3 | 4)
        || data.len() != width as usize * height as usize * components as usize
    {
        return Err(AssetError::invalid("invalid thumbnail dimensions or pixels"));
    }
    encode(width, height, components, data, true, 2 * 1024 * 1024)
}

/// LLImageJ2COJ::encodeImpl: reversible for small uploads, otherwise
/// progressive quality layers with an 8:1 final compression ratio.
/// Pixels and codestream rows are top-to-bottom, like the decoder. Callers
/// using DecodedImage must keep its row order when preparing an upload.
pub fn encode_texture_j2k(width: u32, height: u32, components: u8, data: &[u8], lossless: bool) -> Result<Vec<u8>, AssetError> {
    encode(width, height, components, data, lossless, 32 * 1024 * 1024)
}
fn encode(width: u32, height: u32, components: u8, data: &[u8], lossless: bool, maximum: usize) -> Result<Vec<u8>, AssetError> {
    if !(4..=2048).contains(&width)
        || !(4..=2048).contains(&height)
        || !width.is_power_of_two()
        || !height.is_power_of_two()
        || !matches!(components, 3 | 4)
        || data.len() != width as usize * height as usize * components as usize
    {
        return Err(AssetError::invalid("invalid texture dimensions or pixels"));
    }
    let mut settings = openjp2::EncodeParameters::new();
    let params = settings.low_level();
    let layers = if lossless {
        1
    } else {
        match width * height {
            0..=1024 => 2,
            1025..=16384 => 3,
            16385..=262144 => 4,
            262145..=1048576 => 5,
            _ => 6,
        }
    };
    params.tcp_numlayers = layers;
    for i in 0..layers {
        params.tcp_rates[i as usize] = if lossless {
            0.0
        } else {
            8.0 * (1u32 << ((layers - i - 1) * 2)) as f32
        };
    }
    params.cp_disto_alloc = 1;
    params.irreversible = i32::from(!lossless);
    params.tcp_mct = 1;
    params.numresolution = params.numresolution.min(1 + width.min(height).ilog2() as i32);
    let planes = (0..usize::from(components))
        .map(|c| openjp2::CoreImageComp {
            dx: 1,
            dy: 1,
            w: width,
            h: height,
            prec: 8,
            data: data.iter().skip(c).step_by(components.into()).map(|&v| v.into()).collect(),
            ..Default::default()
        })
        .collect();
    let image = openjp2::Image::from_core_image(openjp2::CoreImage {
        x1: width,
        y1: height,
        numcomps: components.into(),
        color_space: openjp2::ColorSpace::Srgb,
        comps: planes,
        ..Default::default()
    });
    // A raw codestream (J2K, not JP2), as the grid expects.
    match image.encode(settings) {
        Ok(out) if out.len() <= maximum => Ok(out),
        _ => Err(AssetError::J2k("image encoding failed")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thumbnails_round_trip_without_losing_color_or_alpha() {
        for components in [3, 4] {
            let pixels: Vec<_> = (0..64 * 64 * usize::from(components)).map(|i| (i % 251) as u8).collect();
            let encoded = encode_thumbnail_j2k(64, 64, components, &pixels).expect("encode");
            let decoded = decode_j2k(&encoded, 0).expect("decode");
            assert_eq!(decoded.components, components);
            assert_eq!(decoded.data, pixels);
        }
        assert!(encode_thumbnail_j2k(64, 64, 4, &[0; 8]).is_err());
        assert!(encode_thumbnail_j2k(1024, 1024, 4, &[]).is_err());
    }
}
