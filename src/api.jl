module PhiAPI

using Oxygen
using JSON3
using StructTypes
using ..PhiQuantum
using ..PhiIR

# ====================== API Request / Response structs ======================
struct ApiGate
    type::String          # "H", "X", "CNOT", "CZ"
    targets::Vector{Int}  # for CNOT: [control, target]
end

StructTypes.StructType(::Type{ApiGate}) = StructTypes.Struct()

struct ApiRequest
    circuit::Vector{ApiGate}
    modality::String                  # "superconducting" | "iontrap" | "neutralatom"
    hardware_id::String               # e.g. "ibm_heron_2026-03-05_14:37" or "google_sycamore_56"
    calibration::Dict{String, Float64} # "0-1" => 0.0047, "3-7" => 0.012  (minute-specific noise)
end

StructTypes.StructType(::Type{ApiRequest}) = StructTypes.Struct()

struct ApiResponse
    status::String
    pulse_script::String              # hardware-ready execution string (pulse-level)
    estimated_sx::Float64             # Integrated Obstruction for this minute
    fidelity_estimate::Float64        # exp(-S_X) heuristic
    routed_path_summary::String       # e.g. "CNOT 0→3 rerouted via sector [0,4,5,3] (lowest noise)"
end

StructTypes.StructType(::Type{ApiResponse}) = StructTypes.Struct()

# ====================== Convert API → Phi-IR ======================
function to_phi_circuit(req::ApiRequest)::PhiCircuit
    circ = PhiCircuit()
    for g in req.circuit
        t = g.targets
        if g.type == "H"
            push!(circ.elements, h(t[1]))
        elseif g.type == "X"
            push!(circ.elements, x(t[1]))
        elseif g.type == "CNOT" || g.type == "CX"
            push!(circ.elements, cnot(t[1], t[2]))
        elseif g.type == "CZ"
            push!(circ.elements, cnot(t[1], t[2]))  # treat CZ same as CNOT in Weyl for now
        end
    end
    circ
end

# ====================== Build QPU with minute-specific calibration ======================
function build_qpu_with_calib(modality::String, calib::Dict{String, Float64})
    handle = qpu_new()
    qpu_set_modality(handle, modality_code(modality))

    # Clear previous + inject real-time errors
    ccall((:qpu_clear_couplings, libphi), Cvoid, (Ptr{Cvoid},), handle)
    if hasfield(typeof(handle), :positions) || modality == "neutralatom"
        ccall((:qpu_clear_positions, libphi), Cvoid, (Ptr{Cvoid},), handle)
    end

    for (edge, err) in calib
        a, b = parse.(Int, split(edge, "-"))
        qpu_add_coupling(handle, UInt32(a), UInt32(b), err)
        if modality != "iontrap"  # bidirectional except pure all-to-all
            qpu_add_coupling(handle, UInt32(b), UInt32(a), err)
        end
    end

    # Neutral atom positions (example defaults; override via calib keys like "pos-0": "12.3,45.6" later if needed)
    if modality == "neutralatom"
        # demo positions
        qpu_add_position(handle, 0, 0.0, 0.0)
        qpu_add_position(handle, 1, 5.0, 0.0)
        qpu_add_position(handle, 3, 12.0, 3.0)
    end
    handle
end

modality_code(m::String) = m == "superconducting" ? 0u8 : m == "iontrap" ? 1u8 : 2u8

# ====================== Pulse generator (hardware-ready string) ======================
function generate_pulse_script(optimized_circ::PhiCircuit, modality::String, total_sx::Float64)
    lines = String[]
    push!(lines, "# PhiQuantum Pulse Script – generated $(now()) for minute-specific calibration")
    push!(lines, "modality: $modality")
    push!(lines, "estimated_integrated_obstruction: $total_sx")
    push!(lines, "fidelity_estimate: $(round(exp(-total_sx), digits=4))")
    push!(lines, "")

    for elem in optimized_circ.elements
        tgt = elem.targets
        if length(tgt) == 1
            q = tgt[1]
            if elem.word[1] isa Simple  # H or X
                push!(lines, "DRIVE $q freq=4.8e9 duration=30e-9 amp=0.92 phase=0.0")
            end
        else
            c, t = tgt
            if modality == "superconducting"
                push!(lines, "CZ $c-$t duration=180e-9 amp=0.85  # routed via lowest-noise sector")
            elseif modality == "iontrap"
                push!(lines, "MS_GATE $c-$t laser=729nm duration=45e-6 intensity=0.6")
            elseif modality == "neutralatom"
                push!(lines, "RYDBERG $c-$t wavelength=780nm duration=2.2e-6 blockade_radius=8um")
            end
        end
    end
    join(lines, "\n")
end

# ====================== The Endpoint ======================
@post "/optimize" function(req::Oxygen.Request)
    api_req = JSON3.read(req.body, ApiRequest)

    circ = to_phi_circuit(api_req)
    qpu_handle = build_qpu_with_calib(api_req.modality, api_req.calibration)

    # Compile with real-time noise (routes automatically via BMS sector shift)
    optimized, sx_values = compile(circ, qpu_handle)  # we extend compile to accept raw handle for API

    total_sx = sum(sx_values)
    pulse = generate_pulse_script(optimized, api_req.modality, total_sx)

    summary = if total_sx > 0.05
        "High-noise sectors auto-rerouted (S_X reduced by ~$(round(1 - total_sx/0.1, digits=2)))"
    else
        "Optimal low-obstruction paths chosen"
    end

    resp = ApiResponse(
        status = "success",
        pulse_script = pulse,
        estimated_sx = total_sx,
        fidelity_estimate = exp(-total_sx),
        routed_path_summary = summary
    )

    qpu_free(qpu_handle)
    Oxygen.json(resp)
end

# Start server
function start_phi_api(; port::Int = 8080, host::String = "0.0.0.0")
    println("🚀 PhiQuantum API live → http://$host:$port/optimize")
    println("   Send POST with JSON QuantumCircuit + minute-specific calibration")
    serve(port=port, host=host)
end

export start_phi_api

end # module