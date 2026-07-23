function lori.load()
    lori.set.window.title("Lori Test Application")
    Shape = lori.new.shape("rectangle", 10, 10)

    Collider = lori.new.collider(Shape, "dynamic")
    Thing = lori.new.thing(Shape, Collider)

    Object = Thing:spawn(0, 0, 0)
end

function lori.update(delta)
    if lori.get.key.state("w") then
        Object:push(0, -delta * 100000000000)
    end
    if lori.get.key.state("s") then
        Object:push(0, delta * 1000)
    end
    if lori.get.key.state("a") then
        Object:push(-delta * 1000, 0)
    end
    if lori.get.key.state("d") then
        Object:push(delta * 1000, 0)
    end
end