function lori.load()
    lori.set.window.resizable(false)
    lori.set.window.title("Lori Function Test")
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
end