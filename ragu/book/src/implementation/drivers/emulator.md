# Emulator

The simplest implementation of [`Driver`][driver-trait] is the
[`Emulator`][emulator-type], which natively executes circuit code without
enforcing constraints. This driver is useful when the correctness of circuit
code execution does not need to be directly checked, because in that case
circuit code is just native code with pointless steps. Instead of reimplementing
algorithms to anticipate the results or structure of in-circuit computations—to
avoid the overhead of constraint enforcement or wire assignment tracking done in
circuit code—the [`Emulator`][emulator-type] driver can be used to avoid as
much of this overhead as possible.

One of the purposes of the design of the `Driver` abstraction in Ragu is to
enable circuit code to be written so that it can be efficiently natively
executed, reducing code. This especially helps with developing recursive proofs
since almost everything performed by the verifier must also be written to be
executed within a circuit as well.

## Modes

The [`Emulator`][emulator-type] operates in two modes:

- **[`Wireless`][wireless-marker]**: The `Wire` type is `()`, so nothing about
  wire assignments is preserved. This mode is parameterized by a
  [`MaybeKind`][maybekind-trait] to indicate witness availability.

- **[`Wired`][wired-marker]**: The `Wire` type is
  [`WiredValue<F>`][wired-value], which tracks the assignments that the
  circuit code's witness generation logic produces. Wired mode always has
  witness availability.

## Constructors

| Constructor | Mode | Wire | Use Case |
|---|---|---|---|
| [`Emulator::counter`][emulator-counter] | `Wireless<Empty, F>` | `()` | Wire counting, static analysis |
| [`Emulator::execute`][emulator-execute] | `Wireless<Always<()>, F>` | `()` | Native witness execution |
| [`Emulator::wireless`][emulator-wireless] | `Wireless<M, F>` | `()` | Generic (parameterized `MaybeKind`) |
| [`Emulator::extractor`][emulator-extractor] | `Wired<F>` | `WiredValue<F>` | Wire extraction |

[`Emulator::wireless`][emulator-wireless] is useful when witness availability
depends on another driver's behavior, such as when invoking an `Emulator` within
generic circuit code.

## Wire Extraction

[`Gadget`][gadget-trait]s can have their wires extracted from an `Emulator` in
`Wired` mode using [`Emulator::wires`][emulator-wires], which
returns a `Vec<F>` of field element assignments.

<!-- Reference-style links -->

[driver-trait]: ragu_core::drivers::Driver
[emulator-type]: ragu_core::drivers::emulator::Emulator
[wireless-marker]: ragu_core::drivers::emulator::Wireless
[wired-marker]: ragu_core::drivers::emulator::Wired
[wired-value]: ragu_core::drivers::emulator::WiredValue
[gadget-trait]: ragu_core::gadgets::Gadget
[maybekind-trait]: ragu_core::maybe::MaybeKind

[emulator-counter]: ragu_core::drivers::emulator::Emulator::counter
[emulator-execute]: ragu_core::drivers::emulator::Emulator::execute
[emulator-wireless]: ragu_core::drivers::emulator::Emulator::wireless
[emulator-extractor]: ragu_core::drivers::emulator::Emulator::extractor
[emulator-wires]: ragu_core::drivers::emulator::Emulator::wires
