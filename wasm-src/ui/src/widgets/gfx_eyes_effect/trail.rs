//! The spirit's trail: small puffs left behind by the moving wisp.

use alloc::vec::Vec;
use spring::gfx::GfxVertexData;
use spring::gl;

use super::Camera;

const TRAIL_PARTICLE_LIFETIME: f32 = 24.0;
const MAX_TRAIL_PARTICLES: usize = 64;
/// Each puff is a fan: a centre and a closed ring of this many sides.
const PUFF_SIDES: usize = 10;

#[derive(Clone, Copy)]
struct TrailParticle {
    position: spring::Float3,
    velocity: spring::Float3,
    life: f32,
    seed: f32,
}

/// `gfx.shape` serializes fixed-width vertex attributes regardless of the
/// `has_*` flags. Keep those buffers populated even for the untextured,
/// unlit particle fan; empty normal/UV buffers trap inside the generated ABI.
fn particle_vertex() -> GfxVertexData {
    GfxVertexData {
        vertex: vec![0.0; 3],
        normal: vec![0.0, 1.0, 0.0],
        tex_coord: vec![0.0; 2],
        color: vec![0.0; 4],
        has_vertex: true,
        has_normal: false,
        has_tex_coord: false,
        has_color: true,
    }
}

fn set_vertex(vertex: &mut GfxVertexData, point: spring::Float3, color: [f32; 4]) {
    vertex.vertex.copy_from_slice(&[point.x, point.y, point.z]);
    vertex.color.copy_from_slice(&color);
}

#[derive(Default)]
pub struct Trail {
    particles: Vec<TrailParticle>,
    last_position: Option<spring::Float3>,
    /// One puff's fan, reused so drawing allocates nothing.
    vertices: Vec<GfxVertexData>,
}

impl Trail {
    pub fn clear(&mut self) {
        self.particles.clear();
        self.last_position = None;
    }

    /// Advance the particles and emit new ones behind the wisp at `position`.
    pub fn update(&mut self, position: spring::Float3, frame: i32) {
        let (motion_x, motion_z, motion_length) = self
            .last_position
            .replace(position)
            .map(|previous| {
                let x = position.x - previous.x;
                let z = position.z - previous.z;
                (x, z, libm::sqrtf(x * x + z * z))
            })
            .unwrap_or((0.0, 0.0, 0.0));

        // Emit from the body, rather than from the end of a ribbon. The little
        // translucent puffs inherit some backwards velocity, so they read as
        // fragments being left behind by the moving wisp. Their deterministic
        // sinusoidal offsets provide the noisy, uneven silhouette without a
        // random-number dependency in the render widget.
        {
            let particles = &mut self.particles;
            for particle in particles.iter_mut() {
                particle.position.x += particle.velocity.x;
                particle.position.y += particle.velocity.y;
                particle.position.z += particle.velocity.z;
                particle.velocity.x *= 0.96;
                particle.velocity.y = particle.velocity.y * 0.96 + 0.02;
                particle.velocity.z *= 0.96;
                particle.life -= 1.0;
            }
            particles.retain(|particle| particle.life > 0.0);
            if motion_length > 0.25 {
                let movement_x = -motion_x / motion_length;
                let movement_z = -motion_z / motion_length;
                let emission_count = if motion_length > 2.0 { 3 } else { 2 };
                for emission in 0..emission_count {
                    if particles.len() >= MAX_TRAIL_PARTICLES {
                        particles.remove(0);
                    }
                    let seed = frame as f32 * 0.173 + emission as f32 * 2.417;
                    let side = libm::sinf(seed * 1.71) * 7.0;
                    let lift = 8.0 + libm::cosf(seed * 1.13) * 7.0;
                    particles.push(TrailParticle {
                        position: spring::Float3::new(
                            position.x + side * movement_z,
                            position.y + 13.0 + lift,
                            position.z - side * movement_x,
                        ),
                        velocity: spring::Float3::new(
                            movement_x * (0.9 + emission as f32 * 0.25)
                                + libm::cosf(seed * 2.11) * 0.45,
                            0.08 + libm::sinf(seed * 1.37).abs() * 0.12,
                            movement_z * (0.9 + emission as f32 * 0.25)
                                + libm::sinf(seed * 2.11) * 0.45,
                        ),
                        life: TRAIL_PARTICLE_LIFETIME - emission as f32 * 3.0,
                        seed,
                    });
                }
            }
        }
    }

    /// Draw small irregular camera-facing puffs at the particle positions.
    /// Each puff is a separate fan because the low-level shape call has no
    /// primitive-restart support. The pool stays deliberately small so this
    /// remains cheap even while the player is moving continuously.
    pub fn draw(&mut self, camera: &Camera, frame: i32) {
        if self.particles.is_empty() {
            return;
        }
        let matrix = &camera.modelview;
        let right = spring::Float3::new(matrix[0], matrix[4], matrix[8]);
        let up = spring::Float3::new(matrix[1], matrix[5], matrix[9]);
        if self.vertices.is_empty() {
            self.vertices = (0..PUFF_SIDES + 2).map(|_| particle_vertex()).collect();
        }
        let _ = spring::gfx::push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions {
            enable: true,
            set_func: false,
            func: 0,
        });
        let _ = spring::gfx::blending(true);
        let _ = spring::gfx::blend_func(gl::SRC_ALPHA, gl::ONE);
        let _ = spring::gfx::use_shader(0);

        for particle in &self.particles {
            let fade = (particle.life / TRAIL_PARTICLE_LIFETIME).clamp(0.0, 1.0);
            let shimmer = 0.78 + 0.22 * libm::sinf(frame as f32 * 0.18 + particle.seed);
            let size = 3.5 + 3.0 * fade + 1.5 * shimmer;
            let center_alpha = 0.32 * fade * shimmer;
            let edge_alpha = 0.025 * fade;
            set_vertex(
                &mut self.vertices[0],
                particle.position,
                [0.63, 0.72, 1.0, center_alpha],
            );
            for side in 0..=PUFF_SIDES {
                let angle = side as f32 * core::f32::consts::TAU / PUFF_SIDES as f32;
                let noise = 0.72 + 0.28 * libm::sinf(particle.seed + angle * 3.0);
                let offset_x = libm::cosf(angle) * size * noise;
                let offset_y = libm::sinf(angle) * size * noise;
                let point = spring::Float3::new(
                    particle.position.x + right.x * offset_x + up.x * offset_y,
                    particle.position.y + right.y * offset_x + up.y * offset_y,
                    particle.position.z + right.z * offset_x + up.z * offset_y,
                );
                set_vertex(
                    &mut self.vertices[side + 1],
                    point,
                    [0.32, 0.43, 1.0, edge_alpha],
                );
            }
            let _ = spring::gfx::shape(gl::TRIANGLE_FAN, &self.vertices);
        }
        let _ = spring::gfx::pop_attrib(0);
    }
}

#[cfg(test)]
mod tests {
    use super::particle_vertex;

    #[test]
    fn particle_vertices_have_fixed_width_shape_attributes() {
        let vertex = particle_vertex();

        assert_eq!(vertex.vertex.len(), 3);
        assert_eq!(vertex.normal.len(), 3);
        assert_eq!(vertex.tex_coord.len(), 2);
        assert_eq!(vertex.color.len(), 4);
    }
}
