# Copyright (c) 2026 Zeta Reticula Inc
# Licensed under the MIT License. See LICENSE for details.

module PhiIR

export PhiElement, PhiCircuit, h, cnot, x

abstract type AbstractWeylGen end
struct Affine <: AbstractWeylGen; idx::UInt32; end
struct Simple <: AbstractWeylGen; idx::UInt32; end

struct PhiElement
    word::Vector{AbstractWeylGen}
    targets::Vector{UInt32}
end

struct PhiCircuit
    elements::Vector{PhiElement}
end

PhiCircuit() = PhiCircuit(PhiElement[])

h(q::Integer) = PhiElement([Simple(q)], [q])
cnot(c::Integer, t::Integer) = PhiElement([Simple(c), Affine(t), Simple(c)], [c, t])
x(q::Integer) = PhiElement([Simple(q), Simple(q)], [q])

end # module