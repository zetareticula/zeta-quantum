function compile(circuit::PhiCircuit, qpu::SuperconductingQPU)
    println("Φ-Compiler: Affine-Weyl IR → noise-weighted routing...")

    handle = qpu_new()
    try
        # populate coupling as before...
        for (a, neigh) in qpu.coupling_map
            for (b, err) in neigh
                qpu_add_coupling(handle, UInt32(a), UInt32(b), err)
            end
        end

        mapping = collect(0:length(unique(Iterators.flatten(e.targets for e in circuit.elements))) - 1)
        sx_values = Float64[]

        optimized = PhiCircuit()
        for elem in circuit.elements
            sx = ccall((:phi_circuit_obstruction, libphi), Float64,
                       (Ptr{Cvoid}, Ptr{Cvoid}, Ptr{UInt32}, Csize_t),
                       handle, pointer_from_objref(elem), mapping, length(mapping))  # naive – real FFI later
            push!(sx_values, sx)

            # routing logic using previous SWAP insertion on the chosen low-obstruction path...
            # (reuse your earlier dynamic mapping code)
            push!(optimized.elements, elem)  # for now; real optimized word later
        end
        return optimized, sx_values
    finally
        qpu_free(handle)
    end
end