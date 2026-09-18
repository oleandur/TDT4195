// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, color: &Vec<f32>) -> u32 {
    // Implement me!

    // Also, feel free to delete comments :)

    // This should:
    // * Generate a VAO and bind it
    let mut vao_id: u32 = 0;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    // * Generate a VBO and bind it
    let mut vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_id);

    // * Fill it with data
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(vertices), pointer_to_array(vertices), gl::STATIC_DRAW);

    // * Configure a VAP for the data and enable it
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

    gl::EnableVertexAttribArray(0);

    let mut color_vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut color_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, color_vbo_id);

    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(color), pointer_to_array(color), gl::STATIC_DRAW);

    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());

    gl::EnableVertexAttribArray(1);


    // * Generate a IBO and bind it
    let mut ibo_id: u32 = 0;
    gl::GenBuffers(1, &mut ibo_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);

    // * Fill it with data
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(indices), pointer_to_array(indices), gl::STATIC_DRAW);

    // * Return the ID of the VAO
    vao_id
}


fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here

        // 5 triangles
        let vertices: Vec<f32> = vec![
            // Triangle 1
            -0.15,  0.5, -0.4,
            -0.65, -0.4, -0.4,
             0.35, -0.4, -0.4,

            // Triangle 2
             0.0,   0.5, 0.0,
            -0.5,  -0.4, 0.0,
             0.5,  -0.4, 0.0,

            // Triangle 3
             0.15,  0.5, 0.4,
            -0.35, -0.4, 0.4,
             0.65, -0.4, 0.4,
            

            // Triangle 4
            /* 0.0, -0.75, 0.0,
            0.175, -0.25, 0.0,
            -0.175, -0.25, 0.0,
            

            // Triangle 5
            0.725, -0.75, 0.0,
            0.90, -0.25, 0.0,
            0.55, -0.25, 0.0,
             */
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 
            3, 4, 5,
            6, 7, 8,
            /* 9, 10, 11,
            12, 13, 14, */
        ];

        let colors: Vec<f32> = vec![
            // r,  g,   b,   a
            1.0, 0.0, 0.0, 0.7,
            1.0, 0.0, 0.0, 0.7,
            1.0, 0.0, 0.0, 0.7,

            0.0, 1.0, 0.0, 0.4,
            0.0, 1.0, 0.0, 0.4,
            0.0, 1.0, 0.0, 0.4,

            0.0, 0.0, 1.0, 0.4,
            0.0, 0.0, 1.0, 0.4,
            0.0, 0.0, 1.0, 0.4,
        ];
        
        // Checkerboard
        /* let vertices: Vec<f32> = vec![
            5.0, -5.0, -5.0,
            0.0, 5.0, 0.0,
            -5.0, -5.0, 5.0,
        ]; 

        let indices: Vec<u32> = vec![
            0, 1, 2
        ];
         */
         
        // Circle
        /* let segments: u32 = 64;
        let radius: f32 = 0.6;

        let radius_x = radius / window_aspect_ratio;
        let radius_y = radius;

        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        
        
        // Center vertex
        vertices.extend_from_slice(&[
            0.0, 0.0, 0.0
        ]);

        // Vertices around the edge
        for i in 0..segments {
            let angle = (i as f32 / segments as f32)* 2.0 * std::f32::consts::PI;

            let x = radius_x * angle.cos();
            let y = radius_y * angle.sin();

            vertices.extend_from_slice(&[
                x, y, 0.0
            ]);
        }

        // Make triangles from center to neighboring edge vertices
        for i in 1..segments {
            indices.extend_from_slice(&[
                0, i, i + 1,
            ]);
        }

        // Connect
        indices.extend_from_slice(&[
            0, segments, 1,
        ]); */

        // spiral
        let points: u32 = 200;

        /* for i in 0..points {
            let angle = i as f32 * 0.15;
            let radius = i as f32 * 0.0025;

            let x = radius * angle.cos();
            let y = radius * angle.sin();

            vertices.push(x);
            vertices.push(y);
            vertices.push(0.0);

            indices.push(i);
        } */

        // sin

        /* for i in 0..points {
            let t = i as f32 / (points - 1) as f32;

            let x = -0.9 + t * 1.8;
            let y = 0.5 * (t * 2.0 * std::f32::consts::PI).sin();

            vertices.push(x);
            vertices.push(y);
            vertices.push(0.0);

            indices.push(i);
        } */
        

        let my_vao = unsafe {create_vao(&vertices, &indices, &colors)};

        let index_count = indices.len() as i32;

        let demo_vertices: Vec<f32> = vec![
            // x, y, z
            -0.4, -0.3,  1.0,  // Close to camera
            4.0, -3.0, -8.0,  // Far from camera
            0.0,  4.0, -8.0,  // Far from camera
        ];

        let demo_indices: Vec<u32> = vec![
            0, 1, 2
        ];

        let demo_colors: Vec<f32> = vec![
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 0.0, 1.0,
            0.0, 0.0, 1.0, 1.0,
        ];

        let interpolation_vao = unsafe {
            create_vao(
                &demo_vertices,
                &demo_indices,
                &demo_colors,
            )
        };

        let interpolation_count = demo_indices.len() as i32;


        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link()
        };        


        // Used to demonstrate keyboard handling for exercise 2.

        let forward = glm::vec4(0.0, 0.0, -1.0, 0.0);
        let right   = glm::vec4(1.0, 0.0, 0.0, 0.0);
        let up      = glm::vec4(0.0, 1.0, 0.0, 0.0);
        
        let mut camera_x: f32 = 0.0;
        let mut camera_y: f32 = 0.0;
        let mut camera_z: f32 = 3.0;

        let mut camera_yaw: f32 = 0.0;
        let mut camera_pitch: f32 = 0.0;

        let movement_speed: f32 = 2.0;
        let rotation_speed: f32 = 1.5;

        let show_interpolation_demo: bool = true;
        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                // Update camera rotation
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::Left => {
                            camera_yaw += rotation_speed * delta_time;
                        }
                        VirtualKeyCode::Right => {
                            camera_yaw -= rotation_speed * delta_time;
                        }
                        VirtualKeyCode::Up => {
                            camera_pitch += rotation_speed * delta_time;
                        }
                        VirtualKeyCode::Down => {
                            camera_pitch -= rotation_speed * delta_time;
                        }
                        // default handler:
                        _ => { }
                    }
                }
                //Limit rotation vertical
                let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01;
                camera_pitch = camera_pitch.clamp(-pitch_limit, pitch_limit);
                
                // Camera rotation matrix
                let camera_rotation: glm::Mat4 = glm::rotation(camera_yaw, &glm::vec3(0.0, 1.0, 0.0))*glm::rotation(camera_pitch, &glm::vec3(1.0, 0.0, 0.0));

                // Calculate directions relative
                let forward4 = camera_rotation * glm::vec4(0.0, 0.0, -1.0, 0.0);
                let right4 = camera_rotation * glm::vec4(1.0, 0.0, 0.0, 0.0);
                let up4 = camera_rotation * glm::vec4(0.0, 1.0, 0.0, 0.0);

                let forward = glm::vec3(forward4.x, forward4.y, forward4.z);
                let right = glm::vec3(right4.x, right4.y, right4.z);
                let up = glm::vec3(up4.x, up4.y, up4.z);

                // Movement relative
                let mut movement = glm::vec3(0.0, 0.0, 0.0);

                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W => {
                            movement += forward;
                        }
                        VirtualKeyCode::S => {
                            movement -= forward;
                        }
                        VirtualKeyCode::A => {
                            movement -= right;
                        }
                        VirtualKeyCode::D => {
                            movement += right;
                        }
                        VirtualKeyCode::Space => {
                            movement += up;
                        }
                        /* VirtualKeyCode::LShift => {
                            movement -= up;
                        } */
                        _ => {}
                    } 
                }
                // update camera position
                if movement.norm_squared() > 0.0 {
                    let movement = movement.normalize() * movement_speed * delta_time;
                    camera_x += movement.x;
                    camera_y += movement.y;
                    camera_z += movement.z;
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)

            let camera_translation: glm::Mat4 = glm::translation(&glm::vec3(-camera_x, -camera_y, -camera_z));

            let yaw_rotation: glm::Mat4 = glm::rotation(-camera_yaw, &glm::vec3(0.0, 1.0, 0.0));

            let pitch_rotation: glm::Mat4 = glm::rotation(-camera_pitch, &glm::vec3(1.0, 0.0, 0.0));

            let projection: glm::Mat4 = glm::perspective(window_aspect_ratio, 45.0_f32.to_radians(), 1.0, 100.0);

            let transformation: glm::Mat4 = projection * pitch_rotation *yaw_rotation * camera_translation;

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


                // == // Issue the necessary gl:: commands to draw your scene here
                simple_shader.activate();

                gl::UniformMatrix4fv(0, 1, gl::FALSE, transformation.as_ptr());
                
                if show_interpolation_demo {
                    gl::BindVertexArray(interpolation_vao);
                    gl::DrawElements(gl::TRIANGLES, interpolation_count, gl::UNSIGNED_INT, ptr::null());
                } else {
                    gl::BindVertexArray(my_vao);

                    gl::DrawElements(gl::TRIANGLES, index_count, gl::UNSIGNED_INT, ptr::null(), );
                    // gl::DrawElements(gl::LINE_STRIP, index_count, gl::UNSIGNED_INT, ptr::null(), );

                }


            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
