#[macro_use]
extern crate pipeline;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub use pipeline::wgpu_graphics_header::{
    compile_buffer, default_bind_group, generate_swap_chain, graphics_run_indicies,
    graphics_starting_context, setup_render_pass, valid_fragment_shader, valid_vertex_shader,
    BindingPreprocess, GraphicsBindings, GraphicsShader, OutGraphicsBindings, PipelineType,
};

pub use pipeline::shared::{is_gl_builtin, Bindable, Bindings, Context};

pub use pipeline::context::{ready_to_run, update_bind_context, BindingContext};

pub use pipeline::helper::{generate_projection_matrix, generate_view_matrix, load_cube};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    // Create a surface to draw images on
    let surface = wgpu::Surface::create(&window);
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            // Can specify Low/High power usage
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        },
        // Map to Vulkan/Metal/Direct3D 12
        wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    // The device manages the connection and resources of the adapter
    // The queue is a literal queue of tasks for the gpu
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        })
        .await;

    const VERTEXT: (GraphicsShader, BindingContext) = graphics_shader! {
        [[vertex in] vec3] a_position;
        [[vertex in] vec3] vertexColor;
        [[uniform in] mat4] u_view;
        [[uniform in] mat4] u_proj;

        [[out] vec3] fragmentColor;
        [[out] vec4] gl_Position;

        {{
            void main() {
                fragmentColor = vertexColor;
                gl_Position = u_proj * u_view * vec4(0.7 * a_position, 1.0);
            }
        }}
    };

    const FRAGMENT: (GraphicsShader, BindingContext) = graphics_shader! {
        [[in] vec3] fragmentColor;
        [[out] vec4] color;
        {{
            void main() {
                color = vec4(fragmentColor, 1.0);
            }
        }}
    };

    const S_V: GraphicsShader = VERTEXT.0;
    const S_F: GraphicsShader = FRAGMENT.0;
    const VERTEXT_STARTING_BIND_CONTEXT: BindingContext = VERTEXT.1;
    const STARTING_BIND_CONTEXT: BindingContext =
        graphics_starting_context(VERTEXT_STARTING_BIND_CONTEXT, S_F);

    let (program, template_bindings, template_out_bindings, _) =
        compile_valid_graphics_program!(device, S_V, S_F, PipelineType::Color);

    let (positions, _, index_data) = load_cube();

    let color_data = vec![
        [0.583, 0.771, 0.014],
        [0.609, 0.115, 0.436],
        [0.327, 0.483, 0.844],
        [0.822, 0.569, 0.201],
        [0.435, 0.602, 0.223],
        [0.310, 0.747, 0.185],
        [0.597, 0.770, 0.761],
        [0.559, 0.436, 0.730],
        [0.359, 0.583, 0.152],
        [0.483, 0.596, 0.789],
        [0.559, 0.861, 0.639],
        [0.195, 0.548, 0.859],
        [0.014, 0.184, 0.576],
        [0.771, 0.328, 0.970],
        [0.406, 0.615, 0.116],
        [0.676, 0.977, 0.133],
        [0.971, 0.572, 0.833],
        [0.140, 0.616, 0.489],
        [0.997, 0.513, 0.064],
        [0.945, 0.719, 0.592],
        [0.543, 0.021, 0.978],
        [0.279, 0.317, 0.505],
        [0.167, 0.620, 0.077],
        [0.347, 0.857, 0.137],
        [0.055, 0.953, 0.042],
        [0.714, 0.505, 0.345],
        [0.783, 0.290, 0.734],
        [0.722, 0.645, 0.174],
        [0.302, 0.455, 0.848],
        [0.225, 0.587, 0.040],
        [0.517, 0.713, 0.338],
        [0.053, 0.959, 0.120],
        [0.393, 0.621, 0.362],
        [0.673, 0.211, 0.457],
        [0.820, 0.883, 0.371],
        [0.982, 0.099, 0.879],
    ];

    /* let light_direction = vec![[20.0, 0.0, 20.0]];

       let light_ambient = vec![[0.1, 0.0, 0.0]];
    */
    let view_mat = generate_view_matrix();

    let proj_mat = generate_projection_matrix(size.width as f32 / size.height as f32);

    // A "chain" of buffers that we render on to the display
    let mut swap_chain = generate_swap_chain(&surface, &window, &device);

    event_loop.run(move |event, _, control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Everything that can be processed has been so we can now redraw the image on our window
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let mut init_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let mut frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");

                let mut rpass = setup_render_pass(&program, &mut init_encoder, &frame);
                let mut bind_group = default_bind_group(&device);

                let mut bindings: GraphicsBindings = template_bindings.clone();
                let mut out_bindings: OutGraphicsBindings = template_out_bindings.clone();

                let context = Context::new();
                let mut bind_prerun;
                const BIND_CONTEXT_1: BindingContext =
                    update_bind_context(&STARTING_BIND_CONTEXT, "a_position");
                let context1 = bind!(
                    device,
                    bindings,
                    out_bindings,
                    "a_position",
                    positions,
                    context,
                    BIND_CONTEXT_1
                );
                {
                    const BIND_CONTEXT_2: BindingContext =
                        update_bind_context(&BIND_CONTEXT_1, "u_view");
                    let context2 = bind!(
                        device,
                        bindings,
                        out_bindings,
                        "u_view",
                        view_mat,
                        context1,
                        BIND_CONTEXT_2
                    );
                    {
                        const BIND_CONTEXT_3: BindingContext =
                            update_bind_context(&BIND_CONTEXT_2, "vertexColor");
                        let context3 = bind!(
                            device,
                            bindings,
                            out_bindings,
                            "vertexColor",
                            color_data,
                            context2,
                            BIND_CONTEXT_3
                        );

                        {
                            const BIND_CONTEXT_4: BindingContext =
                                update_bind_context(&BIND_CONTEXT_3, "u_proj");
                            let context4 = bind!(
                                device,
                                bindings,
                                out_bindings,
                                "u_proj",
                                proj_mat,
                                context3,
                                BIND_CONTEXT_4
                            );
                            {
                                /*                             const BIND_CONTEXT_5: BindingContext =
                                    update_bind_context!(BIND_CONTEXT_4, "vertexAmbient");
                                bind_vec3(
                                    &program,
                                    &mut bindings,
                                    &mut out_bindings,
                                    &light_ambient,
                                    "vertexAmbient".to_string(),
                                ); */
                                {
                                    /*                                     const BIND_CONTEXT_6: BindingContext = update_bind_context!(
                                        BIND_CONTEXT_5,
                                        "vertexLightDirection"
                                    );
                                    bind_vec3(
                                        &program,
                                        &mut bindings,
                                        &mut out_bindings,
                                        &light_direction,
                                        "vertexLightDirection".to_string(),
                                    ); */
                                    {
                                        const _: () = ready_to_run(BIND_CONTEXT_4);
                                        bind_prerun =
                                            BindingPreprocess::bind(&mut bindings, &out_bindings);
                                        graphics_run_indicies(
                                            &program,
                                            &device,
                                            rpass,
                                            &mut bind_group,
                                            &mut bind_prerun,
                                            &index_data,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                queue.submit(&[init_encoder.finish()]);
            }
            // When the window closes we are done. Change the status
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            // Ignore any other types of events
            _ => {}
        }
    });
}

fn main() {
    // From examples of wgpu-rs, set up a window we can use to view our stuff
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    /* let window = winit::window::Window::new(&event_loop).unwrap(); */

    // Why do we need to be async? Because of event_loop?
    futures::executor::block_on(run(event_loop, window));
}
