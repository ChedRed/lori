# lori


Atrocious naming aside, this Lua library uses Rust's wgpu and winit libraries under the hood, meaning it is extremely fast and capable. At the same time, much of the hard programming is abstracted away into simple Lua functions such as ``lori.set.window.size(..)`` or ``lori.delete.object(..)``. Additionally, Rust's rapier2d library is used to handle physics, which has much better performance in complicated scenarios than box2d!

This project has just begun, and I'm hoping it would become a viable alternative to LOVE which, although I do love it, uses the slower OpenGL backend and Box2D physics engine.

## Functions

Functions with '=' or '-' are implemented, with '-' meaning untested.
```
[=] lori.set.window.title(text: string) -> nil
[-] lori.set.window.size(w: int, h: int) -> nil
[=] lori.set.window.resizable(is: bool) -> nil

[] lori.Object.set.position(x: float, y: float) -> nil
[] lori.Object.set.angle(r: float) -> nil

[] lori.Object.get.position() -> table[x: float, y: float]
[] lori.Object.get.angle() -> float

[=] lori.get.window.size() -> table[w: int, h: int]

[] lori.get.key.state(key: string) -> bool
[] lori.get.mouse.position() -> table[x: float, y: float]

[] lori.new.shape(type: string("rectangle"|"triangle"), x: float, y: float, w: float, h: float) -> lori.Shape
[] lori.new.mesh(vertices: Vertex[], indices: int[]|nil) -> lori.Shape
[] lori.new.image(img: string) -> lori.Shape

[] lori.new.collider(shape: lori.Shape, collision: string("static"|"diaxial"|"dynamic")) -> lori.Collider

[] lori.new.object(x: float, y: float, r: float, shape: lori.Shape, collider: lori.Collider | nil) -> lori.Object
[] lori.new.border(points: Point[]) -> lori.Border

[] lori.new.force(x: float, y: float, fx: float, fy: float) -> lori.Force

[] lori.draw.shape(shape: lori.Shape) -> nil
[] lori.draw.border(border: lori.Border) -> nil
[] lori.draw.line(x1: float, y1: float, x2: float, y2: float, color: number[]) -> nil
[] lori.draw.circle(x: float, y: float, radius: float, color: number[]) -> nil
[] lori.draw.rect(x: float, y: float, w: float, h: float, r: float, color: number[]) -> nil

[] lori.Object.push(force: lori.Force) -> nil

[] lori.delete.object(object: lori.Object) -> nil
```

```
[=] lori.load() -> nil
[=] lori.keypressed(key) -> nil
[=] lori.keyreleased(key) -> nil
[=] lori.mousepressed(x, y, button) -> nil
[=] lori.mousereleased(x, y, button) -> nil
[=] lori.mousemoved(x, y) -> nil
[=] lori.mousescrolled(x, y) -> nil
[=] lori.update() -> nil
[=] lori.render() -> nil
```

```
[] Force
[] Point
[] Shape
[] Border
[] Object
[] Vertex
[] Collider
```

```
TODO:
- Enforce at least one physics tick before rendering, unless lori.update is not present
- Add the rest of the functions ([=] and [-] means fully implemented, but [-] is untested/able)
- Allow lori functions to be optional (eg. lori.load, lori.update, etc)
- Switch to single-threaded, with rapier2d physics thread-fanning
```