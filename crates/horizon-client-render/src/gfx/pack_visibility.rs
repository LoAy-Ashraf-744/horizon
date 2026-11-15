//! Pack visibility controls for the native fallback, retaining the native HDR body passes.
use super::{Frame, Renderer, pack_sky};
use horizon_shaders::SkyVisibility;

impl Renderer {
    pub(super) fn pack_sky_visibility(&self, active: bool) -> SkyVisibility {
        self.pack_post
            .as_ref()
            .filter(|_| active)
            .and_then(|p| p.1.as_ref())
            .map_or_else(SkyVisibility::default, |p| p.sky_visibility())
    }

    pub(super) fn draw_visible_sky_disc(&self, pass: &mut wgpu::RenderPass<'_>) {
        match self.core_sky.as_ref().filter(|c| c.draws_disc()) {
            Some(core) => {
                core.draw_disc(pass);
                pass.set_bind_group(0, &self.group, &[]);
            }
            None => {
                pass.set_pipeline(&self.sky_base);
                pass.draw(0..3, 0..1);
            }
        }
    }

    pub(super) fn draw_visible_bodies(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        frame: &Frame<'_>,
        vertices: u32,
        hdr_gain: bool,
        visibility: SkyVisibility,
    ) {
        // The Overworld's sun is first; a native pack drawing its own skips it.
        let own_sun = !frame.end_sky
            && self
                .native
                .as_ref()
                .is_some_and(|n| n.sky.is_some() && !n.vanilla_sun);
        // Keep main's HDR sun treatment and resource-pack core/sky exception.
        let lifted = hdr_gain
            && self.hdr.as_ref().is_some_and(|h| h.lighting)
            && !self.core_sky.as_ref().is_some_and(|c| c.draws_disc());
        pass.set_vertex_buffer(0, self.celestial_buffer.slice(..));
        for range in pack_sky::celestial_ranges(vertices, frame.end_sky, own_sun, visibility) {
            self.celestial.draw_bodies(pass, lifted, range);
        }
    }
}
