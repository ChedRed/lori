# lori


Atrocious naming aside, this Lua library uses Rust's wgpu and winit libraries under the hood, meaning it is extremely fast and capable. At the same time, much of the hard programming is abstracted away into simple Lua functions such as ``lori.set.window.size(..)`` or ``lori.delete.object(..)``. Additionally, Rust's rapier2d library is used to handle physics, which has much better performance in complicated scenarios than box2d!

This project has just begun, and I'm hoping it would become a viable alternative to LOVE which, although I do love it, uses the slower OpenGL backend and Box2D physics engine.

## Functions

None of these functions have been implemented.
```
lori.set.window.title(text: string) -> None // lori.load, Any (Applied after lori.draw)
lori.set.window.size(w: int, h: int) -> None // lori.load, Any (Applied after lori.draw)
lori.set.window.resizable(is: bool) -> None // lori.load, Any (Applied after lori.draw)

lori.Object.set.position(x: float, y: float) -> None
lori.Object.set.angle(r: float) -> None

lori.Object.get.position() -> table[x: float, y: float]
lori.Object.get.angle() -> float

lori.get.window.size() -> table[w: int, h: int] // lori.


lori.new.shape(type: string("rectangle"|"triangle"), x: float, y: float, w: float, h: float) -> lori.Shape
lori.new.mesh(list[(x: float, y: float, ux: float, uy: float, r: float, g: float, b: float, a: float)]) -> lori.Shape
lori.new.image(img: string) -> lori.Shape

lori.new.collider(shape: lori.Shape, collision: string("static"|"diaxial"|"dynamic")) -> lori.Collider

lori.new.object(x: float, y: float, r: angle, prim: lori.Shape, collider: lori.Collider | None) -> lori.Object
lori.new.border(points: list[x: float, y: float]) -> lori.Border

lori.new.force(x: float, y: float, fx: float, fy: float) -> table[x: float, y: float, fx: float, fy: float]

lori.draw.shape(shape: lori.Shape) -> None
lori.draw.border(border: lori.Border) -> None
lori.draw.primitive(prim: string("circle"|"rectangle"|"triangle")) -> None
lori.draw.line(x1: float, y1: float, x2: float, y2: float) -> None

lori.push.object(object: lori.Object, force: table[x: float, y: float, fx: float, fy: float]) -> None
lori.delete.object(object: lori.Object) -> None
```