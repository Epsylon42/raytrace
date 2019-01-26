extern crate nalgebra as na;
extern crate ncollide3d as nc;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod fb;
mod material;
mod raytrace;
mod scene;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn trace_wasm(scene: &str) -> Result<js_sys::Uint8Array, JsValue> {
    trace(scene)
        .map_err(|err| {
            js_sys::JsString::from(err).into()
        })
        .map(|result| {
            let arr = js_sys::Uint8Array::new_with_length(result.len() as u32);
            let view = js_sys::DataView::new(&arr.buffer(), 0, result.len());
            for (i, byte) in result.into_iter().enumerate() {
                view.set_uint8(i, byte);
            }

            arr
        })
}


pub fn trace(scene: &str) -> Result<Vec<u8>, String> {
    let scene = ron::de::from_str::<scene::Scene>(scene).map_err(|e| format!("{:?}", e))?;

    let size = scene.size;
    let fov = scene.fov;
    let steps = scene.steps;
    let multisample = scene.multisample;

    let (camera, objects, lights) = scene.unpack();

    const SAMPLES: u16 = 2;

    let result = if multisample {
        let fb = raytrace::raytrace(
            (size.0 * SAMPLES, size.1 * SAMPLES),
            fov,
            steps,
            camera,
            objects,
            lights,
        );
        fb::Fb::from_func(size.0, size.1, |x, y| {
            (0..SAMPLES)
                .flat_map(|dx| (0..SAMPLES).map(move |dy| (x * SAMPLES + dx, y * SAMPLES + dy)))
                .map(|(x, y)| fb.get(x, y) * (SAMPLES as f32).powi(2).recip())
                .fold(fb::Color::black(), |acc, a| acc + a)
        })
            .to_image()
    } else {
        raytrace::raytrace(size, fov, steps, camera, objects, lights).to_image()
    };

    let width = result.width();
    let height = result.height();

    let mut buf = Vec::new();
    let mut writer = std::io::Cursor::new(&mut buf);
    image::png::PNGEncoder::new(writer)
        .encode(&result.into_raw(), width, height, image::ColorType::RGB(8)).unwrap();

    Ok(buf)
}
