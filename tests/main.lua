local wheel_position = { 0, 0 }

function lori.load()
    Shapesd = lori.new.shape("rectangle", 100, 150)
    Shapesd:test("Tested!")
end

function lori.update(delta)
    if lori.get.key.state("w") then
        wheel_position[2] = wheel_position[2] - delta * 1000
    end
    if lori.get.key.state("s") then
        wheel_position[2] = wheel_position[2] + delta * 1000
    end
    if lori.get.key.state("a") then
        wheel_position[1] = wheel_position[1] - delta * 1000
    end
    if lori.get.key.state("d") then
        wheel_position[1] = wheel_position[1] + delta * 1000
    end
end

function lori.render()
    -- lori.draw.shape(200, 400, Shape)
    lori.draw.circle(wheel_position[1], wheel_position[2], 200, { 0.5, 0.75, 0, 1 })
    lori.draw.line(100, 100, 200, 300, 10, { 0, 0.5, 1, 1 })
end