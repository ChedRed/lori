-- lori_meta.lua
-- DO NOT require this at runtime. Only for LuaLS / IDE.

--- @class Vertex
--- @field x number
--- @field y number
--- @field u number
--- @field v number
--- @field r number
--- @field g number
--- @field b number
--- @field a number
Vertex = {}

--- @class Point
--- @field x number
--- @field y number
Point = {}

--- @class Color
--- @field r number
--- @field g number
--- @field b number
--- @field a number
Color = {}

--- @class Force
--- @field x number
--- @field y number
--- @field fx number
--- @field fy number
Force = {}

--- @class Shape
Shape = {}

--- @class Collider
Collider = {}

--- @class Border
Border = {}

--- @class Object
Object = {
    set = {
        --- @param x number
        --- @param y number
        --- @return nil
        position = function(x, y) end,
        --- @param r number
        --- @return nil
        angle = function(r) end,
    },
    get = {
        --- @return number
        --- @return number
        position = function() return 0, 0 end,
        --- @return number
        angle = function() return 0 end,
    },
    --- @param force Force
    --- @return nil
    push = function(force) end,
    --- @return nil
    delete = function() end,
}

--- @class Sound
Sound = {
    --- @param volume number
    --- @param pitch number
    --- @return nil
    play = function(volume, pitch) end,
    --- @param count number
    --- @param volume number
    --- @param pitch number
    loop = function(count, volume, pitch) end,
    --- @return nil
    stop = function() end,
}

--- @class Lori
Lori = {
    --- @return nil
    load = function() end,
    --- @param key string
    --- @return nil
    keypressed = function(key) end,
    --- @param key string
    --- @return nil
    keyreleased = function(key) end,
    --- @param x number
    --- @param y number
    --- @param button integer
    --- @return nil
    mousepressed = function(x, y, button) end,
    --- @param x number
    --- @param y number
    --- @param button integer
    --- @return nil
    mousereleased = function(x, y, button) end,
    --- @param x number
    --- @param y number
    --- @return nil
    mousemoved = function(x, y) end,
    --- @param x number
    --- @param y number
    --- @return nil
    mousescrolled = function(x, y) end,
    --- @param delta number
    --- @return nil
    update = function(delta) end,
    --- @return nil
    render = function() end,

    set = {
        window = {
            --- @param text string
            --- @return nil
            title = function(text) end,
            --- @param x integer
            --- @param y integer
            --- @return nil
            size = function(x, y) end,
            --- @param is boolean
            --- @return nil
            resizable = function(is) end,
        }
    },

    get = {
        window = {
            --- @return number
            --- @return number
            size = function() return 0, 0 end,
        },
        key = {
            --- @param key string
            --- @return boolean
            state = function(key) return true end,
        },
        mouse = {
            --- @return number[]
            position = function() return {0, 0} end,
        }
    },

    new = {
        --- @param type "rectangle"|"triangle"
        --- @param x number
        --- @param y number
        --- @param w number
        --- @param h number
        --- @return Shape
        shape = function(type, x, y, w, h) return Shape end,
        --- @param vertices Vertex[]
        --- @param indices integer[] | nil
        --- @return Shape
        mesh = function(vertices, indices) return Shape end,
        --- @param img string
        --- @return Shape
        image = function(img) return Shape end,
        --- @param shape Shape
        --- @param collision "static"|"diaxial"|"dynamic"
        --- @return Collider
        collider = function(shape, collision) return Collider end,
        --- @param x number
        --- @param y number
        --- @param r number
        --- @param shape Shape
        --- @param collider Collider | nil
        --- @return Object
        object = function(x, y, r, shape, collider) return Object end,
        --- @param points Point[]
        --- @return Border
        border = function(points) return Border end,
        --- @param x number
        --- @param y number
        --- @param fx number
        --- @param fy number
        --- @return Force
        force = function(x, y, fx, fy) return Force end,
        --- @param sound string
        --- @return Sound
        sound = function(sound) return Sound end,
    },

    draw = {
        --- @param shape Shape
        --- @return nil
        shape = function(shape) end,
        --- @param border Border
        --- @return nil
        border = function(border) end,
        --- @param x1 number
        --- @param y1 number
        --- @param x2 number
        --- @param y2 number
        --- @param radius number
        --- @param color number[]
        --- @return nil
        line = function(x1, y1, x2, y2, radius, color) end,
        --- @param x number
        --- @param y number
        --- @param radius number
        --- @param color number[]
        --- @return nil
        circle = function(x, y, radius, color) end,
        --- @param x number
        --- @param y number
        --- @param w number
        --- @param h number
        --- @param r number
        --- @param color number[]
        --- @return nil
        rect = function(x, y, w, h, r, color) end
    }
}

--- @diagnostic disable-next-line
lori = Lori