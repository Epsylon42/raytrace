extern crate nalgebra as na;
extern crate ncollide3d as nc;

extern crate rtlib;

// fn open_display(size: (u16, u16), ev: &glium::glutin::EventsLoop) -> (glium::Display, glium::Program) {
//     let mut window = glium::glutin::WindowBuilder::new()
//         .with_dimensions(size.into())
//         .with_title("Raytracer");
//     let context = glium::glutin::ContextBuilder::new();

//     let display = glium::Display::new(window, context, &ev).unwrap();
//     let program = glium::Program::from_source(
//         &display,
//         "
// #version 150

// in vec2 vertex;
// out vec2 tex;

// void main() {
//     tex = vertex / 2.0 + vec2(0.5);
//     gl_Position = vec4(vertex, 0.0, 1.0);
// }
// ",
//         "
// #version 150

// in vec2 tex;
// uniform sampler2D image;

// void main() {
//     gl_FragColor = texture(tex, image);
// }
// ",
//         None
//     ).unwrap();
// }

fn main() {
    let path = std::env::args().nth(1).expect("Expected path to scene");

    let src = std::fs::read_to_string(path).unwrap();
    let result = rtlib::trace(&src).unwrap();

    std::fs::write("result.png", result).unwrap();
}
