// import the deviceExtension trait so we can add buffers to our device

use image::GenericImageView;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    //color: [f32; 3], // using texture will change to only two f32s
    tex_coords: [f32;2]
}

impl Vertex {
    // this expresses how the buffer actually maps data, helps pipeline know what to do with it
    // note there is a macro that helps reduce the verbosity but this is actually nice to see how everything gets laid out
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // how wide a vertex is, to read ext vertex it skips this amount
            step_mode: wgpu::InputStepMode::Vertex, // how frequently it should move between vertices
            attributes: &[
                // the parts of our vertex
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // this is what the location is we look for this data in the shader
                    format: wgpu::VertexFormat::Float3, // corresponds to a vec3
                },
                // have to skip by the length of the first attribute for our offset
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}
/*
// this is for when we have a color specified per vertex
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];
*/
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.99240386], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.56958646], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.050602943], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.15267089], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.7347359], }, // E
];

// create a list of indices also
const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    color: [f64; 3],

    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    num_indices: u32,
    index_buffer: wgpu::Buffer,
    // now the bind group stuff for our texture
    diffuse_bind_group: wgpu::BindGroup,

    diffuse_texture: texture::Texture,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // backend bit points to one of the graphics apis
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        // windows are surfaces I suppose
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        // an adapter is the reference to the gpu lets us create the device and the queeue
        // using the adapter
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // they say this is the trace path?
            )
            .await
            .unwrap();
        // create the swapchain , this is the seeqeuence of buffers that get pushed t othe screen
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT, // usage is how we will use the underlyig textures, RENDER_ATTACHMENT means we draw to the screen
            format: adapter.get_swap_chain_preferred_format(&surface), // how these textures wil be stored on the gpu, displays differ, let the adapter figure it out
            width: size.width, // use the inner_size that we got at the begining
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // how the screen should update from the swap chain,
        };
        // build a swap_chain from the description
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let diffuse_bytes = include_bytes!("../assets/tree.png");
        let diffuse_texture = texture::Texture::from_bytes(&device,&queue,diffuse_bytes,"tree.png texture").unwrap();


        // a bind group is a way to cerate a set of resources that the shader can access
        // start with a layouot then make a group
        // its like a collection of uniforms
        // how do these bindings relate to the layout locationss that come later
        // !! they relate to setting up uniforms and uniform buffers! recall how touch designer does passing samplers to glsl materials and such 
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // entry 0 is the sampled texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    // entry 1 is the sampler itself
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        }, // they really don't explain whhat these things control
                        count: None,
                    },
                ],
                label: Some("bind group layout")
            });
            // apparently we can swap out bindgroups on the fly as long as the share the same descriptions
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse bind group "),
        });

        // make the shader pipeline
        // load the shader code
        let vs_src = include_str!("shader.vert");
        let fs_src = include_str!("shader.frag");

        /*
        // this is needed if we don't compile the shaders ahead of time
        // compile it
        let mut compiler= shaderc::Compiler::new().unwrap();
        // entrypoints "main" mean that we call these functions inside the shader when we load
        let vs_spirv = compiler.compile_into_spirv(vs_src, shaderc::ShaderKind::Vertex,"shader.vert","main", None).unwrap();
        let fs_spirv = compiler.compile_into_spirv(fs_src,shaderc::ShaderKind::Fragment,"shader.frag","main", None).unwrap();
        // get them as data
        let vs_data = wgpu::util::make_spirv(vs_spirv.as_binary_u8());
        let fs_data = wgpu::util::make_spirv(fs_spirv.as_binary_u8());
        */
        // attach the program as a module
        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));
        // make the pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("renedr pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        // make the pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                // this isn't option so not wrapped in a Some
                module: &vs_module,
                entry_point: "main", // this is what function will get called in the shader
                buffers: &[Vertex::desc()], // empty because we are specifying the vertices in the vert shsader for now
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // decide whether triangle faces forward with counter clock wise
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back, // triangles not facing froward get removed
                // have to mess with features if you don't want this
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            // not explained in great detail but has to do with multisampling
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0, // means use all samples
                // antialiasing setting
                alpha_to_coverage_enabled: false,
            },
        });

        // setup the vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            // contents need to be a &[u8] this converts our structs
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        // setup the index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        // return a Self
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            color: [0.0; 3],
            render_pipeline,
            num_indices: INDICES.len() as u32,
            vertex_buffer,
            index_buffer,
            diffuse_bind_group,
            diffuse_texture
        }
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // update the self windows parameters
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        // remake the swapchain
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }
    // no capture info yet
    fn input(&mut self, event: &WindowEvent) -> bool {
        // return true when done processing, and the event won't get called any more
        // match event
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.color = [
                    position.x as f64 / self.size.width as f64,
                    position.y as f64 / self.size.height as f64,
                    0.0,
                ];
                true
            }
            _ => false,
        }
    }
    fn update(&mut self) {}
    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // get a frame to render to
        let frame = self.swap_chain.get_current_frame()?.output;
        // make a command encoder for sending commands to the gpu
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // use encoder to make a render pass, this is the thing thtaht has all the drawing capability
        // I guess we have to put this in a scope also
        // reason being that setting render pass to  encoder.begin_render_pass borrows encoder mutably, but it needs to still exist for the encoder.finish()
        // could use drop(render_pass) but this also works
        {
            // make it mutable so we can use it
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                // describes where we are going to draw our color
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.color[0],
                            g: self.color[1],
                            b: self.color[2],
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None, // what is a depth stencil?
            });

            // do things with the pipeline
            render_pass.set_pipeline(&self.render_pipeline);

            // set the bind group
            // the first argument associates with the first number in our layout(set=0, binding = 0 or 1) uniform texture for our fragment
            render_pass.set_bind_group(0,&self.diffuse_bind_group,&[]);

            // set the vertex buffer, what slot to use for this buffer.
            // interesting! so how do the locations compare to the slots?
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // use the pipeline to draw stuff, first is number of vertices and the second is instance count
            // this is for when you don't use index arrays
            //render_pass.draw(0..self.num_vertices as u32, 0..1);

            // what are each of these arguments?
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        // pass anything that implements iter for our queue
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // no idea about this line with the futures
    use futures::executor::block_on;

    // apparentnly this takes something async and blocks till we've got it
    let mut state: State = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    // size change events
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size); // why the pointer?
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size)
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            // use the update on our state
            state.update();
            // then use render
            match state.render() {
                Ok(_) => {} // nothing bad happened, we are fine
                Err(wgpu::SwapChainError::Lost) => state.resize(state.size), // lets us recreate the swap chain
                // quit if we run out of memory
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // for other errors just print them out
                Err(e) => eprintln!("prob {:?}", e),
            };
        }
        // ensures redraw gets requested again and again
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
