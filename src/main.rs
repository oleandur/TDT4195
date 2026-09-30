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
mod mesh;
mod scene_graph;
mod toolbox;

use scene_graph::SceneNode;
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
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, color: &Vec<f32>, normals: &Vec<f32>) -> u32 {

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

    // Colors
    let mut color_vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut color_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, color_vbo_id);

    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(color), pointer_to_array(color), gl::STATIC_DRAW);

    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());

    gl::EnableVertexAttribArray(1);

    // Normals

    let mut normal_vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut normal_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, normal_vbo_id);

    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(normals), pointer_to_array(normals), gl::STATIC_DRAW);
    gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

    gl::EnableVertexAttribArray(2);


    // * Generate a IBO and bind it
    let mut ibo_id: u32 = 0;
    gl::GenBuffers(1, &mut ibo_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);

    // * Fill it with data
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(indices), pointer_to_array(indices), gl::STATIC_DRAW);

    // * Return the ID of the VAO
    vao_id
}

unsafe fn create_mesh_vao(mesh: &mesh::Mesh) -> u32 {
    create_vao(&mesh.vertices, &mesh.indices, &mesh.colors, &mesh.normals)
}

// A3 task 2b
fn build_helicopter(vaos: &[(u32, i32)]) -> scene_graph::Node {
    let mut body = SceneNode::from_vao(vaos[0].0, vaos[0].1);
    let mut main_rotor = SceneNode::from_vao(vaos[1].0, vaos[1].1);
    let mut tail_rotor = SceneNode::from_vao(vaos[2].0, vaos[2].1);
    let mut door = SceneNode::from_vao(vaos[3].0, vaos[3].1);

    tail_rotor.reference_point = glm::vec3(0.35, 2.3, 10.4);

    body.add_child(&main_rotor);
    body.add_child(&tail_rotor);
    body.add_child(&door);
    body
}

unsafe fn draw_scene(
    node: &scene_graph::SceneNode,
    view_projection_matrix: &glm::Mat4,
    transformation_so_far: &glm::Mat4,
) {
    let translation = glm::translation(&node.position);

    let translate_to_ref = glm::translation(&node.reference_point);

    let translate_from_ref = glm::translation(&glm::vec3(-node.reference_point.x, -node.reference_point.y, -node.reference_point.z));

    let rotation_x = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0));
    let rotation_y = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0));
    let rotation_z = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0));

    let scaling = glm::scaling(&node.scale);

    let local_transform = translation * translate_to_ref * rotation_x * rotation_y * rotation_z * scaling * translate_from_ref;

    let model_matrix = transformation_so_far * local_transform;
    let mvp = view_projection_matrix * model_matrix;

    if node.index_count > 0 {
        gl::UniformMatrix4fv(0, 1, gl::FALSE, mvp.as_ptr());
        gl::UniformMatrix4fv(1, 1, gl::FALSE, model_matrix.as_ptr());

        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, ptr::null());
    }

    for &child in &node.children {
        draw_scene(&*child, view_projection_matrix, &model_matrix);
    }
}

// A3 task 4
fn animate_helicopter(helicopter: &mut SceneNode, time: f32) {
    const ROTOR_SPEED: f32 = 20.0;
    const FLIGHT_HEIGHT: f32 = 10.0;

    helicopter[0].rotation.y = time * ROTOR_SPEED;
    helicopter[1].rotation.x = time * ROTOR_SPEED;

    let heading = toolbox::simple_heading_animation(time);
    helicopter.position.x = heading.x;
    helicopter.position.y = FLIGHT_HEIGHT;
    helicopter.position.z = heading.z;
    helicopter.rotation.x = heading.pitch;
    helicopter.rotation.y = heading.yaw;
    helicopter.rotation.z = heading.roll;
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

        // Load models
        let resources = concat!(env!("CARGO_MANIFEST_DIR"), "/resources");
        let terrain_mesh = mesh::Terrain::load(&format!("{}/lunarsurface.obj", resources));
        let helicopter_mesh = mesh::Helicopter::load(&format!("{}/helicopter.obj", resources));

        let terrain_vao = unsafe { create_mesh_vao(&terrain_mesh) };
        let helicopter_vaos: Vec<(u32, i32)> = (0..4)
            .map(|i| (unsafe { create_mesh_vao(&helicopter_mesh[i]) }, helicopter_mesh[i].index_count))
            .collect();

        let mut scene_root = SceneNode::new();
        let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_mesh.index_count);
        let mut helicopter_root = build_helicopter(&helicopter_vaos);

        terrain_node.add_child(&helicopter_root);
        scene_root.add_child(&terrain_node);
        scene_root.print();

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

        let movement_speed: f32 = 50.0;
        let rotation_speed: f32 = 1.5;

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

            let projection: glm::Mat4 = glm::perspective(window_aspect_ratio, 45.0_f32.to_radians(), 1.0, 1000.0);

            let transformation: glm::Mat4 = projection * pitch_rotation *yaw_rotation * camera_translation;

            animate_helicopter(&mut helicopter_root, elapsed);

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


                // == // Issue the necessary gl:: commands to draw your scene here
                simple_shader.activate();

                let identity: glm::Mat4 = glm::identity();
                draw_scene(&scene_root, &transformation, &identity);

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
