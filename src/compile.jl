# Copyright (c) 2026 Zeta Reticula Inc
# Licensed under the MIT License. See LICENSE for details.

# Compile function for Superconducting QPU
function compile(circuit::PhiCircuit, qpu::SuperconductingQPU)
    # Print compilation message
    println("Φ-Compiler: Affine-Weyl IR → noise-weighted routing...")

    # Create QPU handle
    handle = qpu_new()
    # Try to populate coupling map
    try
        # populate coupling as before...
        for (a, neigh) in qpu.coupling_map
            # Print coupling map
            println("Coupling: $a -> $neigh")
            # Add coupling to QPU
            for (b, err) in neigh # Iterate over neighbors
                # Print coupling
                println("Adding coupling: $a -> $b")
                # Add coupling
                qpu_add_coupling(handle, UInt32(a), UInt32(b), err) # Add coupling to QPU
            end
        end

        # Create mapping and sx values
        mapping = collect(0:length(unique(Iterators.flatten(e.targets for e in circuit.elements))) - 1)
        # Initialize sx values
        sx_values = Float64[]

        # Optimize circuit
        optimized = PhiCircuit()
        # For every element in the circuit, calculate obstruction and add to optimized circuit
        for elem in circuit.elements
            # Calculate obstruction
            sx = ccall((:phi_circuit_obstruction, libphi), Float64,
                       (Ptr{Cvoid}, Ptr{Cvoid}, Ptr{UInt32}, Csize_t),
                       handle, pointer_from_objref(elem), mapping, length(mapping))  # naive – real FFI later
            # Add obstruction to sx values
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

# Compile function for Photonic QPU
function compile(circuit::PhiCircuit, qpu::PhotonicQPU)
    # Print compilation message
    println("Φ-Compiler: Affine-Weyl IR → noise-weighted routing...")
    
    # Photonic QPU has linear nearest-neighbor topology with beam splitter errors
    handle = qpu_new()
    try
        # Set modality to photonic
        qpu_set_modality(handle, 3)  # Photonic modality code
        
        # Populate coupling map for linear optical network
        for (a, neigh) in qpu.coupling_map
            for (b, err) in neigh
                qpu_add_coupling(handle, UInt32(a), UInt32(b), err)
            end
        end
        
        # Photonic circuits have limited two-qubit gates (probabilistic)
        # Route through beam splitter network with loss considerations
        sx_values = Float64[]
        optimized = PhiCircuit()
        
        for elem in circuit.elements
            if length(elem.targets) == 1
                # Single-qubit gates are deterministic in photonics
                push!(optimized.elements, elem)
                push!(sx_values, 0.0)
            else
                # Two-qubit gates require heralded success or fusion
                c, t = elem.targets[1], elem.targets[2]
                sx = qpu_gate_cost(handle, UInt32(c), UInt32(t))
                push!(sx_values, sx)
                push!(optimized.elements, elem)
            end
        end
        
        return optimized, sx_values
    finally
        qpu_free(handle)
    end
end

