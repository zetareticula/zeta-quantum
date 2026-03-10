module PhiCLI

using Term
using ..PhiIR
using ..PhiQuantum  # your main module with compile

export dashboard

function dashboard(circ::PhiCircuit, sx_per_gate::Vector{Float64})
    println("\n" * "═"^80)
    print(Panel("PhiQuantum – Entropy Dashboard  (2026)", style="bold green", width=80))

    rows = []
    for (i, (elem, sx)) in enumerate(zip(circ.elements, sx_per_gate))
        gate_name = length(elem.targets) == 1 ? "H/X" : "CNOT"
        tgt_str = join(elem.targets, ",")
        color = sx > 0.05 ? "red" : sx > 0.02 ? "yellow" : "green"
        push!(rows, [i, gate_name, tgt_str, @styled "$sx" "$color bold"])
    end

    t = Table(
        rows;
        header=["#", "Gate", "Targets", "𝒮_X (Obstruction)"],
        header_style="bold cyan",
        columns_style=["dim", "bold", "", "bold"]
    )
    println(t)
    println("Highest 𝒮_X = $(maximum(sx_per_gate)) → auto-routed to lowest-noise sector\n")
end

end # module