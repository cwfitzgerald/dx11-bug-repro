use pollster::block_on;
use std::ptr;
use wgpu::*;

fn main() {
    let mut rd = renderdoc::RenderDoc::<renderdoc::V141>::new().ok();

    wgpu_subscriber::initialize_default_subscriber(None);

    let instance = Instance::new(BackendBit::DX12);

    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();

    let (device, queue) =
        block_on(adapter.request_device(&DeviceDescriptor::default(), None)).unwrap();

    if let Some(ref mut rd) = rd {
        rd.start_frame_capture(ptr::null(), ptr::null());
    }

    let vert = device.create_shader_module(include_spirv!("../shaders/color.vert.spv"));
    let frag = device.create_shader_module(include_spirv!("../shaders/color.frag.spv"));

    let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bgl"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::StorageTexture {
                dimension: TextureViewDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                readonly: true,
            },
            count: None,
        }],
    });

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("tex"),
        size: Extent3d {
            width: 1,
            height: 1,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsage::COPY_DST | TextureUsage::STORAGE,
    });

    queue.write_texture(
        TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
        },
        &[128, 128, 0, 0],
        TextureDataLayout {
            offset: 0,
            bytes_per_row: 4,
            rows_per_image: 0,
        },
        Extent3d {
            width: 1,
            height: 1,
            depth: 1,
        },
    );

    let bg1 = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bg1"),
        layout: &bgl,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(
                &texture.create_view(&TextureViewDescriptor::default()),
            ),
        }],
    });

    let pll = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline layout"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let pl = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("pipeline"),
        layout: Some(&pll),
        vertex_stage: ProgrammableStageDescriptor {
            module: &vert,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &frag,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor::default()),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Rgba8UnormSrgb,
            alpha_blend: Default::default(),
            color_blend: Default::default(),
            write_mask: Default::default(),
        }],
        depth_stencil_state: None,
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
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

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("encoder"),
    });

    let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[RenderPassColorAttachmentDescriptor {
            attachment: &render_texture_view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
        }],
        depth_stencil_attachment: None,
    });

    rpass.set_pipeline(&pl);
    rpass.set_bind_group(0, &bg1, &[]);
    rpass.draw(0..3, 0..1);

    drop(rpass);

    queue.submit(Some(encoder.finish()));

    device.poll(Maintain::Wait);

    if let Some(ref mut rd) = rd {
        rd.end_frame_capture(ptr::null(), ptr::null());
    }
}
