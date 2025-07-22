#[rustler::nif]
fn add(a: i64, b: i64) -> i64 {
    a + b 
}
use rustler::{Binary, Env, Error, NewBinary, Term};
use std::io::Cursor;

// --- NIF for Resizing PNG ---
#[rustler::nif(schedule = "DirtyCpu")]
fn resize_png<'a>(env: Env<'a>, png_data: Binary<'a>, max_height: u32) -> Result<Term<'a>, Error> {
    // 1. Load image from memory
    let img = match image::load_from_memory(png_data.as_slice()) {
        Ok(i) => i,
        Err(e) => return Ok((rustler::types::atom::error(), format!("Image loading error: {}", e)).encode(env)),
    };

    let (width, height) = img.dimensions();

    // 2. Only resize if the image is taller than the max height
    if height <= max_height {
        // No resize needed, return the original binary
        return Ok((rustler::types::atom::ok(), png_data.to_term(env)).encode(env));
    }

    // 3. Calculate new dimensions preserving aspect ratio
    let aspect_ratio = width as f32 / height as f32;
    let new_height = max_height;
    let new_width = (new_height as f32 * aspect_ratio).round() as u32;

    // 4. Resize using a high-quality filter
    let resized_img = image::imageops::resize(&img, new_width, new_height, image::imageops::FilterType::Lanczos3);

    // 5. Encode the resized image back to a PNG in a new buffer
    let mut result_buf = Vec::new();
    if let Err(e) = resized_img.write_to(&mut Cursor::new(&mut result_buf), image::ImageOutputFormat::Png) {
        return Ok((rustler::types::atom::error(), format!("PNG encoding error: {}", e)).encode(env));
    }

    // 6. Return the new PNG data as an Elixir binary
    let mut binary = NewBinary::new(env, result_buf.len());
    binary.as_mut_slice().copy_from_slice(&result_buf);

    Ok((rustler::types::atom::ok(), binary.into()).encode(env))
}


// --- NIF for PNG Minification ---
#[rustler::nif(schedule = "DirtyCpu")]
fn minify_png<'a>(env: Env<'a>, png_data: Binary<'a>, level: u8) -> Result<Term<'a>, Error> {
    let clamped_level = level.min(6);
    let mut options = oxipng::Options::from_preset(clamped_level);
    options.force = true;

    match oxipng::optimize_from_memory(&png_data, &options) {
        Ok(optimized_data) => {
            let mut binary = NewBinary::new(env, optimized_data.len());
            binary.as_mut_slice().copy_from_slice(&optimized_data);
            Ok((rustler::types::atom::ok(), binary.into()).encode(env))
        }
        Err(e) => {
            let error_reason = e.to_string();
            Ok((rustler::types::atom::error(), error_reason).encode(env))
        }
    }
}

// --- NIF for PNG to WebP Conversion ---
#[rustler::nif]
fn png_to_webp<'a>(env: Env<'a>, png_data: Binary<'a>, quality: f32) -> Result<Term<'a>, Error> {
    let decoder = png::Decoder::new(Cursor::new(png_data.as_slice()));
    let mut reader = match decoder.read_info() {
        Ok(r) => r,
        Err(e) => return Ok((rustler::types::atom::error(), format!("PNG decoding error: {}", e)).encode(env)),
    };
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = match reader.next_frame(&mut buf) {
        Ok(i) => i,
        Err(e) => return Ok((rustler::types::atom::error(), format!("PNG frame reading error: {}", e)).encode(env)),
    };
    let bytes = &buf[..info.buffer_size()];
    let encoder = webp::Encoder::from_rgba(bytes, info.width, info.height);
    let webp_memory = encoder.encode(quality);
    let mut binary = NewBinary::new(env, webp_memory.len());
    binary.as_mut_slice().copy_from_slice(&webp_memory);
    Ok((rustler::types::atom::ok(), binary.into()).encode(env))
}

// Register all NIFs with Elixir
rustler::init!("Elixir.Imagewise.ImageProcessor.OxiPNG", 
    [add, resize_png, minify_png, png_to_webp]
    );
