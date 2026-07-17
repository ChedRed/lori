local mouse_position = { 0, 0 }
local wheel_velocity = { 0, 0 }
local wheel_position = { 0, 0 }

function lori.load()
    -- lori.set.window.resizable(false)
    -- lori.set.window.title("Lori Function Test")
end

function lori.keypressed(key)
    print("KEY PRESSED", key)
end

function lori.keyreleased(key)
    print("KEY RELEASED", key)
end

function lori.mousepressed(x, y, button)
    print("MOUSE PRESSED", x, y, button)
    mouse_position = {x, y}
end

function lori.mousereleased(x, y, button)
    print("MOUSE RELEASED", x, y, button)
end

function lori.mousemoved(x, y)
    print("MOUSE MOVED", x, y)
end

function lori.mousescrolled(x, y)
    print("MOUSE SCROLLED", x, y)
end

function lori.update()
    wheel_position[1] = wheel_position[1] + wheel_velocity[1]
    wheel_position[2] = wheel_position[2] + wheel_velocity[2]
    wheel_velocity[1] = wheel_velocity[1] * 0.9
    wheel_velocity[2] = wheel_velocity[2] * 0.9
end

function lori.render()
    if lori.get.key.state("w") then
        wheel_velocity[2] = wheel_velocity[2] - 1
    end
    if lori.get.key.state("s") then
        wheel_velocity[2] = wheel_velocity[2] + 1
    end
    if lori.get.key.state("a") then
        wheel_velocity[1] = wheel_velocity[1] - 1
    end
    if lori.get.key.state("d") then
        wheel_velocity[1] = wheel_velocity[1] + 1
    end

    local x, y = lori.get.window.size()
    lori.draw.rect(mouse_position[1] - 100, mouse_position[2] - 100, 200, 200, 0, {0, 1, 1, 1});
    lori.draw.circle(wheel_position[1], wheel_position[2], 200, {0.5, 0.75, 0, 1});
end