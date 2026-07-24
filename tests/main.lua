function lori.load()
    lori.set.window.title("Lori Test Application")
    PlayerShape = lori.new.shape("rectangle", 32, 32, { 1., 0., 0., 1. })

    PlayerCollider = lori.new.collider(PlayerShape, "dynamic")
    PlayerSpawner = lori.new.spawner(PlayerShape, PlayerCollider)

    Object = PlayerSpawner:spawn(0, 0, 0)

    lori.set.gravity(0, -1000)
end

function lori.update(delta)
    if lori.get.key.state("w") then
        Object:move(0, 10)
    end
    if lori.get.key.state("s") then
        Object:move(0, -10)
    end
    if lori.get.key.state("a") then
        Object:move(-10, 0)
    end
    if lori.get.key.state("d") then
        Object:move(10, 0)
    end
end