# lori
LORI is a game engine that is a Lua Overlay, and has a Rust Interior!

Atrocious naming aside, this Lua library uses Rust's wgpu and winit libraries under the hood, meaning it is extremely fast and capable. At the same time, much of the hard programming is abstracted away into simple Lua functions such as ``lori.create.window(..)`` or ``lori.move(..)``. Additionally, Rust's rapier2d library is used to handle physics, which has much better performance in complicated scenarios than box2d!

This project has just begun, and I'm hoping it would become a viable alternative to LOVE which, although I do love it, uses the slower OpenGL backend.