use log::trace;
use web_sys::OffscreenCanvas;
use wgpu::{Backends, Instance, InstanceDescriptor, RequestAdapterOptions};

pub async fn list_adapters(surface: OffscreenCanvas) {
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::BROWSER_WEBGPU,
        dx12_shader_compiler: Default::default(),
    });

    let surface = instance
        .create_surface_from_offscreen_canvas(surface)
        .expect("surface");

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("adapter");
    trace!(
        "        [server]: found a wgpu adapter: {:?}",
        adapter.get_info()
    );
}
