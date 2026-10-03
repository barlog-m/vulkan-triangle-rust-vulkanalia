use ash::vk;
use glam::f32::{Vec2, Vec3};

use crate::vk_models::Vertex;
use crate::vk_utils::{vk_create_index_buffer, vk_create_vertex_buffer};

pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    vertex_buffer_alloc: vk_mem::Allocation,
    pub index_buffer: vk::Buffer,
    index_buffer_alloc: vk_mem::Allocation,

    pub indices_count: u32,
    pub vertices_count: u32,
}

impl Mesh {
    pub fn new(
        device: &ash::Device,
        allocator: &vk_mem::Allocator,
        graphics_queue: &vk::Queue,
        command_pool: &vk::CommandPool,
    ) -> Self {
        let indices = [0, 1, 2, 2, 3, 0];

        let (index_buffer, index_buffer_alloc, indices_count) =
            vk_create_index_buffer(&device, &allocator, graphics_queue, command_pool, &indices);

        let vertices = [
            Vertex {
                position: Vec2 { x: -0.5, y: -0.5 },
                color: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            },
            Vertex {
                position: Vec2 { x: 0.5, y: -0.5 },
                color: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
            },
            Vertex {
                position: Vec2 { x: 0.5, y: 0.5 },
                color: Vec3 { x: 0.0, y: 0.0, z: 1.0 },
            },
            Vertex {
                position: Vec2 { x: -0.5, y: 0.5 },
                color: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
            },
        ];

        let (vertex_buffer, vertex_buffer_alloc, vertices_count) =
            vk_create_vertex_buffer(&device, &allocator, graphics_queue, command_pool, &vertices);

        Self {
            vertex_buffer,
            vertex_buffer_alloc,
            index_buffer,
            index_buffer_alloc,
            indices_count,
            vertices_count,
        }
    }

    pub fn destroy(&mut self, device: &ash::Device, allocator: &vk_mem::Allocator) {
        unsafe {
            let _ = device.device_wait_idle();
            allocator.destroy_buffer(self.vertex_buffer, &mut self.vertex_buffer_alloc);
            allocator.destroy_buffer(self.index_buffer, &mut self.index_buffer_alloc);
        }
    }
}
