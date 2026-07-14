# lori


Atrocious naming aside, this Lua library uses Rust's wgpu and winit libraries under the hood, meaning it is extremely fast and capable. At the same time, much of the hard programming is abstracted away into simple Lua functions such as ``lori.create.window(..)`` or ``lori.move(..)``. Additionally, Rust's rapier2d library is used to handle physics, which has much better performance in complicated scenarios than box2d!

This project has just begun, and I'm hoping it would become a viable alternative to LOVE which, although I do love it, uses the slower OpenGL backend and Box2D physics engine.

## Functions

None of these functions have been implemented.
```
lori.set.window.title(title: string) -> None
lori.set.window.size(w: int, h: int) -> None
lori.get.window.size() -> table[w: int, h: int]

lori.new.primitive(type: string, x: float, y: float) -> lori.Shape
lori.new.mesh(list[(x: float, y: float, ux: float, uy: float, r: float, g: float, b: float, a: float)]) -> lori.Shape
lori.new.image(img: string, x: float, y: float) -> lori.Shape

lori.new.object(x: float, y: float, r: angle, prim: lori.Shape) -> lori.Object
lori.new.border(points: list[x: float, y: float]) -> lori.Border

lori.new.force(x: float, y: float, fx: float, fy: float) -> table[x: float, y: float, fx: float, fy: float]

lori.push.object(object: lori.Object, force: table[x: float, y: float, fx: float, fy: float]) -> None
lori.delete.object()
```