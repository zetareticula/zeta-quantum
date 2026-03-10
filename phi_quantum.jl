module PhiQuantum

using Libdl

# Load the Rust shared library for high-speed noise-weighted routing
const libphi = "libphi_core.so"

abstract type AbstractQPU end

struct SuperconductingQPU <: AbstractQPU
    # User-provided hardware topology: qubit → list of (neighbor, CNOT error rate)
    # Edges can be one-way; we automatically make them bidirectional in the backend
    coupling_map::Dict{Int, Vector{Tuple{Int, Float64}}}
end

struct IonTrapQPU <: AbstractQPU end

struct QuantumCircuit
    gates::Vector{Symbol}
    targets::Vector{Vector{Int}}
end

# ====================== Rust FFI Wrappers ======================
function qpu_new()::Ptr{Cvoid}
    ccall((:qpu_new, libphi), Ptr{Cvoid}, ())
end

function qpu_add_coupling(handle::Ptr{Cvoid}, a::UInt32, b::UInt32, error_rate::Float64)
    ccall((:qpu_add_coupling, libphi), Cvoid,
          (Ptr{Cvoid}, UInt32, UInt32, Float64),
          handle, a, b, error_rate)
end

function qpu_find_optimal_path(handle::Ptr{Cvoid}, start::UInt32, end_::UInt32)::Vector{UInt32}
    MAX_PATH = 256
    path_buf = Vector{UInt32}(undef, MAX_PATH)
    len = ccall((:qpu_find_optimal_path, libphi), Csize_t,
                (Ptr{Cvoid}, UInt32, UInt32, Ptr{UInt32}, Csize_t),
                handle, start, end_, pointer(path_buf), MAX_PATH)
    len == 0 ? UInt32[] : path_buf[1:len]
end

function qpu_free(handle::Ptr{Cvoid})
    ccall((:qpu_free, libphi), Cvoid, (Ptr{Cvoid},), handle)
end

# ====================== Unified Compile API ======================
function compile(circuit::QuantumCircuit, qpu::IonTrapQPU)
    println("Optimizing for Ion Trap (All-to-all connectivity)...")
    # No topology constraints → return circuit unchanged
    return circuit
end

function compile(circuit::QuantumCircuit, qpu::SuperconductingQPU)
    println("Optimizing for Superconducting Topology (noise-weighted routing)...")

    # Find number of logical qubits
    n_qubits = 0
    for tgts in circuit.targets
        for t in tgts
            n_qubits = max(n_qubits, t + 1)
        end
    end
    n_qubits = max(n_qubits, 1)

    # Initial placement: logical i ↔ physical i
    mapping  = collect(0:(n_qubits-1))   # logical → current physical
    position = collect(0:(n_qubits-1))   # physical → current logical

    optimized_gates   = Symbol[]
    optimized_targets = Vector{Int}[]

    handle = qpu_new()
    try
        # Populate Rust QPU (auto-bidirectional for typical hardware)
        for (a, neighbors) in qpu.coupling_map
            for (b, err) in neighbors
                qpu_add_coupling(handle, UInt32(a), UInt32(b), err)
                qpu_add_coupling(handle, UInt32(b), UInt32(a), err)
            end
        end

        for (g, tgts) in zip(circuit.gates, circuit.targets)
            if length(tgts) == 1
                # Single-qubit gate: just remap to current physical location
                phys = mapping[tgts[1] + 1]
                push!(optimized_gates, g)
                push!(optimized_targets, [phys])
                continue
            end

            # Two-qubit gate
            if g in (:CNOT, :CX, :CZ) && length(tgts) == 2
                c_log = tgts[1]
                t_log = tgts[2]
                c_phys = mapping[c_log + 1]
                t_phys = mapping[t_log + 1]

                # Find lowest-noise path (BMS sector shift)
                path = qpu_find_optimal_path(handle, UInt32(c_phys), UInt32(t_phys))

                if length(path) < 2
                    error("No path between physical qubits $c_phys and $t_phys")
                end

                # Route target toward control by successive SWAPs along the path
                for i in (length(path)-1):-1:2
                    p1 = Int(path[i-1])   # closer to control
                    p2 = Int(path[i])     # closer to target

                    push!(optimized_gates, :SWAP)
                    push!(optimized_targets, [p1, p2])

                    # Update dynamic mapping
                    l1 = position[p1 + 1]
                    l2 = position[p2 + 1]
                    mapping[l1 + 1] = p2
                    mapping[l2 + 1] = p1
                    position[p1 + 1] = l2
                    position[p2 + 1] = l1
                end

                # Now the two qubits are adjacent → apply original gate on current positions
                current_c = mapping[c_log + 1]
                current_t = mapping[t_log + 1]
                push!(optimized_gates, g)
                push!(optimized_targets, [current_c, current_t])
            else
                # Other multi-qubit gates: remap only (no routing for demo)
                phys_tgts = [mapping[t + 1] for t in tgts]
                push!(optimized_gates, g)
                push!(optimized_targets, phys_tgts)
            end
        end
    finally
        qpu_free(handle)
    end

    return QuantumCircuit(optimized_gates, optimized_targets)
end

export QuantumCircuit, SuperconductingQPU, IonTrapQPU, compile

end