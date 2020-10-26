use pollster::block_on;
use wgpu::*;
use wgpu::util::{DeviceExt, BufferInitDescriptor};

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);

    let instance = Instance::new(BackendBit::DX11);

    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();

    let (device, queue) = block_on(adapter.request_device(&DeviceDescriptor::default(), None)).unwrap();

    let vert = device.create_shader_module(include_spirv!("../shaders/color.vert.spv"));
    let frag = device.create_shader_module(include_spirv!("../shaders/color.frag.spv"));

    let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bgl"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::StorageBuffer {
                dynamic: false,
                min_binding_size: None,
                readonly: false
            },
            count: None
        }]
    });

    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("buffer"),
        contents: bytemuck::bytes_of(&0.5_f32),
        usage: BufferUsage::STORAGE,
    });
    
    let bg = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bg"),
        layout: &bgl,
        entries: &[BindGroupEntry { binding: 0, resource: BindingResource::Buffer(buffer.slice(..)) }]
    });
    
    let pll = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline layout"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[]
    });
    
    let pl = device.create_render_pipeline(&RenderPipelineDescriptor{
        label: Some("pipeline"),
        layout: Some(&pll),
        vertex_stage: ProgrammableStageDescriptor { module: &vert, entry_point: "main" },
        fragment_stage: Some(ProgrammableStageDescriptor { module: &frag, entry_point: "main" }),
        rasterization_state: Some(RasterizationStateDescriptor::default()),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Rgba8UnormSrgb,
            alpha_blend: Default::default(),
            color_blend: Default::default(),
            write_mask: Default::default()
        }],
        depth_stencil_state: None,
        vertex_state: VertexStateDescriptor { index_format: IndexFormat::Uint32, vertex_buffers: &[] },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false
    });

    let render_texture = device.create_texture(&TextureDescriptor {
        label: Some("render output"),

        size: Extent3d {
            width: 100,
            height: 100,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsage::OUTPUT_ATTACHMENT,
    });

    let render_texture_view = render_texture.create_view(&TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("encoder") });

    let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[RenderPassColorAttachmentDescriptor {
            attachment: &render_texture_view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            }
        }],
        depth_stencil_attachment: None,
    });

    rpass.set_pipeline(&pl);
    rpass.set_bind_group(0, &bg, &[]);
    rpass.draw(0..3, 0..1);

    drop(rpass);

    queue.submit(Some(encoder.finish()));

    device.poll(Maintain::Wait);
}
