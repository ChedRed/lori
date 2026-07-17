function lori.load()
    -- lori.set.window.resizable(false)
    -- lori.set.window.title("Lori Function Test")
end

function lori.keypressed(key)
    print("KEY PRESSED", key)
    if key == "e" then
        local x, y = lori.get.window.size()
        print("Window:", x, y)
    end
end

function lori.keyreleased(key)
    print("KEY RELEASED", key)
end

function lori.update()
end

function lori.render()
    local x, y = lori.get.window.size()
    lori.draw.rect(100, 100, 200, 200, 0, {1, 1, 0, 1});
    lori.draw.rect(400, 500, 300, 250, 0, {0, 1, 1, 1});
end