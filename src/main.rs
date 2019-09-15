extern crate nalgebra as na;
extern crate ncollide3d as nc;

extern crate rtlib;

fn main() {
    let path = std::env::args().nth(1).expect("Expected path to scene");

    #[cfg(feature = "update")]
    {
        let mut window: Option<minifb::Window> = None;
        let mut size = (0, 0);
        let mut prev_src = None;

        while window.as_ref().map(|w| w.is_open()).unwrap_or(true) {
            let src = std::fs::read_to_string(&path).unwrap();
            let scene = match ron::de::from_str::<rtlib::scene::Scene>(&src) {
                Ok(scene) => scene,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    continue;
                }
            };

            if Some(src.as_str()) != prev_src.as_ref().map(String::as_str) {
                if scene.size != size || window.is_none() {
                    size = scene.size;
                    window = Some(minifb::Window::new(
                        "Raytrace",
                        size.0 as usize,
                        size.1 as usize,
                        Default::default(),
                    ).unwrap());
                }

                let window = window.as_mut().unwrap();
                let buf = rtlib::trace_scene(scene).to_packed_bgr();
                window.update_with_buffer(&buf).unwrap();
                println!("Update!");
            }
            prev_src = Some(src);
        }
    }
    #[cfg(not(feature = "update"))]
    {
        let src = std::fs::read_to_string(path).unwrap();
        let result = rtlib::trace(&src).unwrap();

        std::fs::write("result.png", result).unwrap();
    }
}
